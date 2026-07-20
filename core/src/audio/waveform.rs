use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    thread,
    time::UNIX_EPOCH,
};

use tempfile::Builder;

use crate::CoreError;

pub const BASE_FRAMES_PER_PEAK: u32 = 16;
pub const MAX_WAVEFORM_PEAKS_PER_REQUEST: u32 = 2_048;

const MAX_WAVEFORM_CACHE_BYTES: usize = 256 * 1024 * 1024;
const SIDECAR_MAGIC: [u8; 8] = *b"KHSWAVE\0";
const SIDECAR_VERSION: u32 = 1;
const MAX_SIDECAR_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub channels: u16,
    pub sample_rate: u32,
    pub duration_seconds: f64,
    pub waveform: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum WaveformPeaks {
    Loading,
    Ready {
        source_version: String,
        frames_per_peak: u32,
        start_peak: u32,
        total_peaks: u32,
        minimums: Vec<f32>,
        maximums: Vec<f32>,
    },
}

#[derive(Debug)]
pub(super) struct WaveformPyramid {
    channels: u16,
    sample_rate: u32,
    duration_seconds: f64,
    levels: Vec<WaveformLevel>,
}

#[derive(Debug)]
struct WaveformLevel {
    frames_per_peak: u32,
    minimums: Vec<f32>,
    maximums: Vec<f32>,
}

#[derive(Debug, Default)]
pub(super) struct WaveformCache {
    entries: HashMap<PathBuf, CachedWaveform>,
    pending: HashMap<PathBuf, PendingWaveform>,
    access_counter: u64,
    memory_bytes: usize,
}

#[derive(Debug)]
struct CachedWaveform {
    fingerprint: SourceFingerprint,
    source_version: String,
    pyramid: Arc<WaveformPyramid>,
    memory_bytes: usize,
    last_access: u64,
}

#[derive(Debug)]
struct PendingWaveform {
    fingerprint: SourceFingerprint,
    receiver: mpsc::Receiver<Result<Arc<WaveformPyramid>, CoreError>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceFingerprint {
    length: u64,
    modified_seconds: u64,
    modified_nanos: u32,
}

impl WaveformPyramid {
    pub(super) fn from_samples(
        channels: u16,
        sample_rate: u32,
        duration_seconds: f64,
        samples: impl IntoIterator<Item = f32>,
    ) -> Option<Self> {
        let channel_count = usize::from(channels);
        if channel_count == 0 || sample_rate == 0 {
            return None;
        }

        let mut minimums = Vec::new();
        let mut maximums = Vec::new();
        let mut frame_minimum = 1.0_f32;
        let mut frame_maximum = -1.0_f32;
        let mut samples_in_frame = 0_usize;
        let mut bucket_minimum = 1.0_f32;
        let mut bucket_maximum = -1.0_f32;
        let mut frames_in_bucket = 0_u32;

        for sample in samples {
            let sample = if sample.is_finite() {
                sample.clamp(-1.0, 1.0)
            } else {
                0.0
            };
            frame_minimum = frame_minimum.min(sample);
            frame_maximum = frame_maximum.max(sample);
            samples_in_frame += 1;
            if samples_in_frame != channel_count {
                continue;
            }

            bucket_minimum = bucket_minimum.min(frame_minimum);
            bucket_maximum = bucket_maximum.max(frame_maximum);
            frames_in_bucket += 1;
            frame_minimum = 1.0;
            frame_maximum = -1.0;
            samples_in_frame = 0;

            if frames_in_bucket == BASE_FRAMES_PER_PEAK {
                minimums.push(bucket_minimum);
                maximums.push(bucket_maximum);
                bucket_minimum = 1.0;
                bucket_maximum = -1.0;
                frames_in_bucket = 0;
            }
        }

        if frames_in_bucket > 0 {
            minimums.push(bucket_minimum);
            maximums.push(bucket_maximum);
        }
        if minimums.is_empty() {
            return None;
        }

        let mut levels = vec![WaveformLevel {
            frames_per_peak: BASE_FRAMES_PER_PEAK,
            minimums,
            maximums,
        }];
        while levels.last().is_some_and(|level| level.minimums.len() > 1) {
            let previous = levels.last().expect("waveform has a base level");
            let Some(frames_per_peak) = previous.frames_per_peak.checked_mul(2) else {
                break;
            };
            let mut minimums = Vec::with_capacity(previous.minimums.len().div_ceil(2));
            let mut maximums = Vec::with_capacity(previous.maximums.len().div_ceil(2));
            for index in (0..previous.minimums.len()).step_by(2) {
                let next = (index + 1).min(previous.minimums.len() - 1);
                minimums.push(previous.minimums[index].min(previous.minimums[next]));
                maximums.push(previous.maximums[index].max(previous.maximums[next]));
            }
            levels.push(WaveformLevel {
                frames_per_peak,
                minimums,
                maximums,
            });
        }

        Some(Self {
            channels,
            sample_rate,
            duration_seconds,
            levels,
        })
    }

    fn peaks(
        &self,
        source_version: String,
        requested_frames_per_peak: u32,
        start_peak: u32,
        peak_count: u32,
    ) -> Result<WaveformPeaks, CoreError> {
        if requested_frames_per_peak == 0 {
            return Err(CoreError::new("frames per waveform peak must be positive"));
        }
        if peak_count == 0 || peak_count > MAX_WAVEFORM_PEAKS_PER_REQUEST {
            return Err(CoreError::new(format!(
                "waveform peak count must be between 1 and {MAX_WAVEFORM_PEAKS_PER_REQUEST}"
            )));
        }

        let level = self
            .levels
            .iter()
            .find(|level| level.frames_per_peak >= requested_frames_per_peak)
            .unwrap_or_else(|| self.levels.last().expect("waveform has at least one level"));
        let total_peaks = u32::try_from(level.minimums.len()).unwrap_or(u32::MAX);
        let start = usize::try_from(start_peak)
            .unwrap_or(usize::MAX)
            .min(level.minimums.len());
        let count = usize::try_from(peak_count).expect("bounded waveform peak count fits usize");
        let end = start.saturating_add(count).min(level.minimums.len());

        Ok(WaveformPeaks::Ready {
            source_version,
            frames_per_peak: level.frames_per_peak,
            start_peak: u32::try_from(start).unwrap_or(u32::MAX),
            total_peaks,
            minimums: level.minimums[start..end].to_vec(),
            maximums: level.maximums[start..end].to_vec(),
        })
    }

    fn memory_bytes(&self) -> usize {
        self.levels
            .iter()
            .map(|level| (level.minimums.len() + level.maximums.len()) * size_of::<f32>())
            .sum()
    }
}

impl WaveformCache {
    pub(super) fn peaks(
        &mut self,
        path: &Path,
        sidecar_path: Option<&Path>,
        frames_per_peak: u32,
        start_peak: u32,
        peak_count: u32,
    ) -> Result<WaveformPeaks, CoreError> {
        validate_peak_request(frames_per_peak, peak_count)?;
        let fingerprint = source_fingerprint(path)?;
        self.access_counter = self.access_counter.wrapping_add(1);
        if let Some(cached) = self.entries.get_mut(path)
            && cached.fingerprint == fingerprint
        {
            cached.last_access = self.access_counter;
            return cached.pyramid.peaks(
                cached.source_version.clone(),
                frames_per_peak,
                start_peak,
                peak_count,
            );
        }

        if let Some(stale) = self.entries.remove(path) {
            self.memory_bytes = self.memory_bytes.saturating_sub(stale.memory_bytes);
        }
        if self
            .pending
            .get(path)
            .is_some_and(|pending| pending.fingerprint != fingerprint)
        {
            self.pending.remove(path);
        }

        match self
            .pending
            .get(path)
            .map(|pending| pending.receiver.try_recv())
        {
            Some(Ok(result)) => {
                self.pending.remove(path);
                let pyramid = result?;
                if source_fingerprint(path)? != fingerprint {
                    return self.start_analysis(path, sidecar_path, source_fingerprint(path)?);
                }
                let memory_bytes = pyramid.memory_bytes();
                let source_version = fingerprint.version();
                self.memory_bytes = self.memory_bytes.saturating_add(memory_bytes);
                self.entries.insert(
                    path.to_owned(),
                    CachedWaveform {
                        fingerprint,
                        source_version: source_version.clone(),
                        pyramid: Arc::clone(&pyramid),
                        memory_bytes,
                        last_access: self.access_counter,
                    },
                );
                self.evict();
                pyramid.peaks(source_version, frames_per_peak, start_peak, peak_count)
            }
            Some(Err(mpsc::TryRecvError::Empty)) => Ok(WaveformPeaks::Loading),
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.pending.remove(path);
                Err(CoreError::new(
                    "waveform analysis worker stopped unexpectedly",
                ))
            }
            None => self.start_analysis(path, sidecar_path, fingerprint),
        }
    }

    fn start_analysis(
        &mut self,
        path: &Path,
        sidecar_path: Option<&Path>,
        fingerprint: SourceFingerprint,
    ) -> Result<WaveformPeaks, CoreError> {
        let source_path = path.to_owned();
        let cache_path = sidecar_path.map(Path::to_owned);
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .name("waveform-analysis".to_owned())
            .spawn(move || {
                let result = load_pyramid(&source_path, cache_path.as_deref(), fingerprint);
                let _ = sender.send(result);
            })
            .map_err(|error| {
                CoreError::new(format!("failed to start waveform analysis: {error}"))
            })?;
        self.pending.insert(
            path.to_owned(),
            PendingWaveform {
                fingerprint,
                receiver,
            },
        );
        Ok(WaveformPeaks::Loading)
    }

    fn evict(&mut self) {
        while self.memory_bytes > MAX_WAVEFORM_CACHE_BYTES && self.entries.len() > 1 {
            let Some(path) = self
                .entries
                .iter()
                .min_by_key(|(_, cached)| cached.last_access)
                .map(|(path, _)| path.clone())
            else {
                break;
            };
            if let Some(cached) = self.entries.remove(&path) {
                self.memory_bytes = self.memory_bytes.saturating_sub(cached.memory_bytes);
            }
        }
    }
}

fn validate_peak_request(frames_per_peak: u32, peak_count: u32) -> Result<(), CoreError> {
    if frames_per_peak == 0 {
        return Err(CoreError::new("frames per waveform peak must be positive"));
    }
    if peak_count == 0 || peak_count > MAX_WAVEFORM_PEAKS_PER_REQUEST {
        return Err(CoreError::new(format!(
            "waveform peak count must be between 1 and {MAX_WAVEFORM_PEAKS_PER_REQUEST}"
        )));
    }
    Ok(())
}

fn load_pyramid(
    path: &Path,
    sidecar_path: Option<&Path>,
    fingerprint: SourceFingerprint,
) -> Result<Arc<WaveformPyramid>, CoreError> {
    let cached_pyramid =
        sidecar_path.and_then(|sidecar_path| read_sidecar(sidecar_path, fingerprint));
    let loaded_sidecar = cached_pyramid.is_some();
    let pyramid = cached_pyramid.map_or_else(|| super::decoder::decode(path), Ok)?;
    if let Some(sidecar_path) = sidecar_path
        && !loaded_sidecar
    {
        let _ = write_sidecar(sidecar_path, fingerprint, &pyramid);
    }
    Ok(pyramid)
}

pub(super) fn sidecar_path(path: &Path) -> PathBuf {
    let mut sidecar_path = path.as_os_str().to_owned();
    sidecar_path.push(".khs-waveform");
    PathBuf::from(sidecar_path)
}

fn read_sidecar(path: &Path, fingerprint: SourceFingerprint) -> Option<Arc<WaveformPyramid>> {
    let file = File::open(path).ok()?;
    let file_length = file.metadata().ok()?.len();
    if file_length > MAX_SIDECAR_BYTES {
        return None;
    }
    let mut reader = BufReader::new(file);
    let mut magic = [0_u8; SIDECAR_MAGIC.len()];
    reader.read_exact(&mut magic).ok()?;
    if magic != SIDECAR_MAGIC
        || read_u32(&mut reader)? != SIDECAR_VERSION
        || read_u64(&mut reader)? != fingerprint.length
        || read_u64(&mut reader)? != fingerprint.modified_seconds
        || read_u32(&mut reader)? != fingerprint.modified_nanos
    {
        return None;
    }

    let channels = read_u16(&mut reader)?;
    let sample_rate = read_u32(&mut reader)?;
    let duration_seconds = f64::from_bits(read_u64(&mut reader)?);
    let level_count = usize::try_from(read_u32(&mut reader)?).ok()?;
    if !(1..=32).contains(&level_count)
        || !(1..=2).contains(&channels)
        || sample_rate == 0
        || !duration_seconds.is_finite()
        || duration_seconds <= 0.0
    {
        return None;
    }

    let mut levels = Vec::with_capacity(level_count);
    for level_index in 0..level_count {
        let frames_per_peak = read_u32(&mut reader)?;
        let peak_count = usize::try_from(read_u32(&mut reader)?).ok()?;
        let expected_frames_per_peak = BASE_FRAMES_PER_PEAK.checked_shl(
            u32::try_from(level_index).expect("bounded waveform level index fits u32"),
        )?;
        let expected_peak_count = levels
            .last()
            .map_or(peak_count, |previous: &WaveformLevel| {
                previous.minimums.len().div_ceil(2)
            });
        let peak_bytes = u64::try_from(peak_count).ok()?.checked_mul(8)?;
        if frames_per_peak != expected_frames_per_peak
            || peak_count == 0
            || (level_index > 0 && peak_count != expected_peak_count)
            || peak_bytes > file_length
        {
            return None;
        }

        let mut minimums = Vec::with_capacity(peak_count);
        let mut maximums = Vec::with_capacity(peak_count);
        for _ in 0..peak_count {
            let value = f32::from_bits(read_u32(&mut reader)?);
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return None;
            }
            minimums.push(value);
        }
        for _ in 0..peak_count {
            let value = f32::from_bits(read_u32(&mut reader)?);
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return None;
            }
            maximums.push(value);
        }
        if minimums
            .iter()
            .zip(&maximums)
            .any(|(minimum, maximum)| minimum > maximum)
        {
            return None;
        }
        levels.push(WaveformLevel {
            frames_per_peak,
            minimums,
            maximums,
        });
    }

    let mut trailing = [0_u8; 1];
    if reader.read(&mut trailing).ok()? != 0 {
        return None;
    }
    Some(Arc::new(WaveformPyramid {
        channels,
        sample_rate,
        duration_seconds,
        levels,
    }))
}

fn write_sidecar(
    path: &Path,
    fingerprint: SourceFingerprint,
    pyramid: &WaveformPyramid,
) -> std::io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("waveform sidecar must have a parent directory"))?;
    let mut temporary = Builder::new()
        .suffix(".khs-waveform.tmp")
        .tempfile_in(parent)?;
    {
        let mut writer = BufWriter::new(temporary.as_file_mut());
        writer.write_all(&SIDECAR_MAGIC)?;
        write_u32(&mut writer, SIDECAR_VERSION)?;
        write_u64(&mut writer, fingerprint.length)?;
        write_u64(&mut writer, fingerprint.modified_seconds)?;
        write_u32(&mut writer, fingerprint.modified_nanos)?;
        write_u16(&mut writer, pyramid.channels)?;
        write_u32(&mut writer, pyramid.sample_rate)?;
        write_u64(&mut writer, pyramid.duration_seconds.to_bits())?;
        write_u32(
            &mut writer,
            u32::try_from(pyramid.levels.len())
                .map_err(|_| std::io::Error::other("too many waveform levels"))?,
        )?;
        for level in &pyramid.levels {
            write_u32(&mut writer, level.frames_per_peak)?;
            write_u32(
                &mut writer,
                u32::try_from(level.minimums.len())
                    .map_err(|_| std::io::Error::other("waveform level is too large"))?,
            )?;
            for value in &level.minimums {
                write_u32(&mut writer, value.to_bits())?;
            }
            for value in &level.maximums {
                write_u32(&mut writer, value.to_bits())?;
            }
        }
        writer.flush()?;
    }
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(std::io::Error::other)?;
    Ok(())
}

fn read_u16(reader: &mut impl Read) -> Option<u16> {
    let mut bytes = [0_u8; size_of::<u16>()];
    reader.read_exact(&mut bytes).ok()?;
    Some(u16::from_le_bytes(bytes))
}

fn read_u32(reader: &mut impl Read) -> Option<u32> {
    let mut bytes = [0_u8; size_of::<u32>()];
    reader.read_exact(&mut bytes).ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> Option<u64> {
    let mut bytes = [0_u8; size_of::<u64>()];
    reader.read_exact(&mut bytes).ok()?;
    Some(u64::from_le_bytes(bytes))
}

fn write_u16(writer: &mut impl Write, value: u16) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn write_u32(writer: &mut impl Write, value: u32) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn write_u64(writer: &mut impl Write, value: u64) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn source_fingerprint(path: &Path) -> Result<SourceFingerprint, CoreError> {
    let metadata = fs::metadata(path).map_err(|error| {
        CoreError::new(format!(
            "failed to inspect audio file {}: {error}",
            path.display()
        ))
    })?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .unwrap_or_default();
    Ok(SourceFingerprint {
        length: metadata.len(),
        modified_seconds: modified.as_secs(),
        modified_nanos: modified.subsec_nanos(),
    })
}

impl SourceFingerprint {
    fn version(self) -> String {
        format!(
            "{}-{}-{}",
            self.length, self.modified_seconds, self.modified_nanos
        )
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn pyramid(frames: &[(f32, f32)]) -> WaveformPyramid {
        WaveformPyramid::from_samples(
            2,
            48_000,
            f64::from(u32::try_from(frames.len()).expect("test waveform length fits u32"))
                / 48_000.0,
            frames.iter().flat_map(|frame| [frame.0, frame.1]),
        )
        .expect("samples should build a waveform")
    }

    #[test]
    fn preserves_signed_extrema_for_every_decoded_frame() {
        let mut frames = vec![(0.1, 0.2); 17];
        frames[4] = (-0.9, 0.4);
        frames[16] = (-0.3, 0.8);

        let waveform = pyramid(&frames);
        let base = &waveform.levels[0];

        assert_eq!(base.minimums, [-0.9, -0.3]);
        assert_eq!(base.maximums, [0.4, 0.8]);
    }

    #[test]
    fn coarser_levels_include_odd_final_buckets() {
        let mut frames = vec![(0.0, 0.0); 16 * 5];
        frames[16 * 4] = (-1.0, 0.75);

        let waveform = pyramid(&frames);

        assert_eq!(waveform.levels[0].minimums.len(), 5);
        assert_eq!(waveform.levels[1].minimums.len(), 3);
        assert_eq!(waveform.levels[2].minimums.len(), 2);
        assert_eq!(waveform.levels[3].minimums, [-1.0]);
        assert_eq!(waveform.levels[3].maximums, [0.75]);
    }

    #[test]
    fn range_queries_are_bounded_and_select_a_matching_lod() {
        let waveform = pyramid(&vec![(-0.5, 0.5); 16 * 16]);

        let peaks = waveform
            .peaks("version".to_owned(), 64, 1, 2)
            .expect("range should be valid");
        let WaveformPeaks::Ready {
            frames_per_peak,
            start_peak,
            total_peaks,
            minimums,
            maximums,
            ..
        } = peaks
        else {
            panic!("in-memory waveform query should be ready");
        };

        assert_eq!(frames_per_peak, 64);
        assert_eq!(start_peak, 1);
        assert_eq!(total_peaks, 4);
        assert_eq!(minimums, [-0.5, -0.5]);
        assert_eq!(maximums, [0.5, 0.5]);
        assert!(
            waveform
                .peaks(
                    "version".to_owned(),
                    16,
                    0,
                    MAX_WAVEFORM_PEAKS_PER_REQUEST + 1,
                )
                .is_err()
        );
    }

    #[test]
    fn sidecars_round_trip_and_reject_changed_sources() {
        let directory = tempdir().expect("temporary directory should be created");
        let source_path = directory.path().join("sample.wav");
        fs::write(&source_path, b"source").expect("source should be written");
        let fingerprint = source_fingerprint(&source_path).expect("source should be inspected");
        let waveform = pyramid(&vec![(-0.75, 0.5); 16 * 4]);
        let cache_path = sidecar_path(&source_path);

        write_sidecar(&cache_path, fingerprint, &waveform)
            .expect("waveform sidecar should be written");
        let restored =
            read_sidecar(&cache_path, fingerprint).expect("matching waveform sidecar should load");

        assert_eq!(restored.channels, waveform.channels);
        assert_eq!(restored.sample_rate, waveform.sample_rate);
        assert_eq!(restored.levels.len(), waveform.levels.len());
        assert_eq!(restored.levels[0].minimums, waveform.levels[0].minimums);
        assert_eq!(restored.levels[0].maximums, waveform.levels[0].maximums);
        fs::write(&source_path, b"changed source").expect("source should be replaced");
        let changed = source_fingerprint(&source_path).expect("changed source should be inspected");
        assert!(read_sidecar(&cache_path, changed).is_none());
    }

    #[test]
    fn sidecar_loading_starts_without_blocking_the_request() {
        let directory = tempdir().expect("temporary directory should be created");
        let source_path = directory.path().join("sample.wav");
        fs::write(&source_path, b"source").expect("source should be written");
        let fingerprint = source_fingerprint(&source_path).expect("source should be inspected");
        let cache_path = sidecar_path(&source_path);
        write_sidecar(
            &cache_path,
            fingerprint,
            &pyramid(&vec![(-0.75, 0.5); 16 * 4]),
        )
        .expect("waveform sidecar should be written");
        let mut cache = WaveformCache::default();

        assert!(matches!(
            cache
                .peaks(&source_path, Some(&cache_path), 16, 0, 4)
                .expect("analysis should start"),
            WaveformPeaks::Loading
        ));
        for _ in 0..100 {
            if matches!(
                cache
                    .peaks(&source_path, Some(&cache_path), 16, 0, 4)
                    .expect("analysis should complete"),
                WaveformPeaks::Ready { .. }
            ) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("background sidecar load did not complete");
    }
}
