import { Sidebar, SidebarContent, SidebarRail } from "@/components/ui/sidebar";

import { ProjectTree } from "./project-tree";

export function ProjectSidebar() {
  return (
    <Sidebar className="absolute h-full" collapsible="offcanvas">
      <SidebarContent className="overflow-hidden">
        <ProjectTree />
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  );
}
