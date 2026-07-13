import { Kbd, KbdGroup } from "@/components/ui/kbd";
import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarTrigger,
} from "@/components/ui/menubar";
import { useAppStore } from "@/stores/app-store";
import { useSettingsStore } from "@/stores/settings-store";

export function AppMenu() {
  const serverStatus = useAppStore((state) => state.serverStatus);
  const workspace = useAppStore((state) => state.workspace);
  const newProject = useAppStore((state) => state.newProject);
  const openProject = useAppStore((state) => state.openProject);
  const saveProject = useAppStore((state) => state.saveProject);
  const saveProjectAs = useAppStore((state) => state.saveProjectAs);
  const undo = useAppStore((state) => state.undo);
  const redo = useAppStore((state) => state.redo);
  const openSettings = useSettingsStore((state) => state.open);
  const unavailable = serverStatus !== "ready";

  return (
    <Menubar className="h-full rounded-none border-0 bg-transparent p-0 shadow-none">
      <MenubarMenu>
        <MenubarTrigger className="h-full rounded-none px-3 text-xs font-normal">
          File
        </MenubarTrigger>
        <MenubarContent>
          <MenubarItem disabled={unavailable} onSelect={() => void newProject()}>
            New project
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>N</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarItem disabled={unavailable} onSelect={() => void openProject()}>
            Open project
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>O</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarSeparator />
          <MenubarItem disabled={!workspace} onSelect={() => void saveProject()}>
            Save
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>S</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarItem disabled={!workspace} onSelect={() => void saveProjectAs()}>
            Save as...
          </MenubarItem>
          <MenubarSeparator />
          <MenubarItem onSelect={() => void window.kickHatSnare.closeWindow()}>Exit</MenubarItem>
        </MenubarContent>
      </MenubarMenu>

      <MenubarMenu>
        <MenubarTrigger className="h-full rounded-none px-3 text-xs font-normal">
          Edit
        </MenubarTrigger>
        <MenubarContent>
          <MenubarItem disabled={!workspace?.history.canUndo} onSelect={() => void undo()}>
            {workspace?.history.undoLabel ? `Undo ${workspace.history.undoLabel}` : "Undo"}
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>Z</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarItem disabled={!workspace?.history.canRedo} onSelect={() => void redo()}>
            {workspace?.history.redoLabel ? `Redo ${workspace.history.redoLabel}` : "Redo"}
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>Shift</Kbd>
              <Kbd>Z</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarSeparator />
          <MenubarItem disabled={unavailable} onSelect={openSettings}>
            Settings...
          </MenubarItem>
        </MenubarContent>
      </MenubarMenu>

      <MenubarMenu>
        <MenubarTrigger className="h-full rounded-none px-3 text-xs font-normal">
          View
        </MenubarTrigger>
        <MenubarContent>
          <MenubarItem disabled>Toggle mixer</MenubarItem>
          <MenubarItem disabled>Toggle browser</MenubarItem>
        </MenubarContent>
      </MenubarMenu>

      <MenubarMenu>
        <MenubarTrigger className="h-full rounded-none px-3 text-xs font-normal">
          Help
        </MenubarTrigger>
        <MenubarContent>
          <MenubarItem disabled>About KickHatSnare</MenubarItem>
        </MenubarContent>
      </MenubarMenu>
    </Menubar>
  );
}
