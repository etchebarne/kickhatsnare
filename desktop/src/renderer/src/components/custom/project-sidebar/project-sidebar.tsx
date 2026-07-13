import { Sidebar, SidebarContent, SidebarRail } from "@/components/ui/sidebar";
import { useAppStore } from "@/stores/app-store";

import { ProjectTree } from "./project-tree";

export function ProjectSidebar() {
  const workspace = useAppStore((state) => state.workspace);
  const paths = workspace
    ? [`${workspace.name}/`, ...workspace.files.map((file) => `${workspace.name}/${file}`)]
    : [];

  return (
    <Sidebar className="absolute h-full" collapsible="offcanvas">
      <SidebarContent className="overflow-hidden">
        <ProjectTree
          key={workspace ? (workspace.projectFilePath ?? `unsaved:${workspace.name}`) : "loading"}
          paths={paths}
          rootPath={workspace?.name ?? ""}
        />
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  );
}
