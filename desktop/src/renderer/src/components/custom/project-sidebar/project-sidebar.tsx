import { Sidebar, SidebarContent, SidebarRail } from "@/components/ui/sidebar";
import { useAppStore } from "@/stores/app-store";

import { ProjectTree } from "./project-tree";

export function ProjectSidebar() {
  const library = useAppStore((state) => state.library);
  const workspace = useAppStore((state) => state.workspace);
  const topLevelRoots = new Set(workspace ? [workspace.name] : []);
  const pinnedCategory = uniqueTreeRoot("Pinned folders", topLevelRoots);
  const usedRoots = new Set<string>();
  const rootPath = workspace?.name ?? "";
  const pinnedRoots =
    library?.pinnedFolders.map((folder) => {
      const name = folder.isAvailable ? folder.name : `${folder.name} (unavailable)`;
      return {
        id: folder.id,
        rootPath: `${pinnedCategory}/${uniqueTreeRoot(name, usedRoots)}`,
        files: folder.files,
      };
    }) ?? [];
  const paths = [
    ...(workspace ? [`${rootPath}/`, ...workspace.files.map((file) => `${rootPath}/${file}`)] : []),
    `${pinnedCategory}/`,
    ...pinnedRoots.flatMap((folder) => [
      `${folder.rootPath}/`,
      ...folder.files.map((file) => `${folder.rootPath}/${file}`),
    ]),
  ];

  return (
    <Sidebar className="absolute h-full" collapsible="offcanvas">
      <SidebarContent className="overflow-hidden">
        <ProjectTree
          key={workspace ? (workspace.projectFilePath ?? `unsaved:${workspace.name}`) : "loading"}
          paths={paths}
          rootPath={rootPath}
          pinnedRoots={pinnedRoots}
          categoryPaths={[pinnedCategory]}
        />
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  );
}

function uniqueTreeRoot(name: string, usedRoots: Set<string>): string {
  let root = name;
  for (let suffix = 2; usedRoots.has(root); suffix += 1) root = `${name} (${suffix})`;
  usedRoots.add(root);
  return root;
}
