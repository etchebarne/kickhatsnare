import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from "@/components/ui/menubar";

export function AppMenu() {
  return (
    <Menubar className="h-full rounded-none border-0 bg-transparent p-0 shadow-none">
      <MenubarMenu>
        <MenubarTrigger className="h-full rounded-none px-3 text-xs font-normal">
          File
        </MenubarTrigger>
        <MenubarContent>
          <MenubarItem disabled>
            New project
            <MenubarShortcut>Ctrl+N</MenubarShortcut>
          </MenubarItem>
          <MenubarItem disabled>
            Open project
            <MenubarShortcut>Ctrl+O</MenubarShortcut>
          </MenubarItem>
          <MenubarSeparator />
          <MenubarItem disabled>
            Save
            <MenubarShortcut>Ctrl+S</MenubarShortcut>
          </MenubarItem>
          <MenubarItem disabled>Save as...</MenubarItem>
          <MenubarSeparator />
          <MenubarItem onSelect={() => void window.kickHatSnare.closeWindow()}>Exit</MenubarItem>
        </MenubarContent>
      </MenubarMenu>

      <MenubarMenu>
        <MenubarTrigger className="h-full rounded-none px-3 text-xs font-normal">
          Edit
        </MenubarTrigger>
        <MenubarContent>
          <MenubarItem disabled>
            Undo
            <MenubarShortcut>Ctrl+Z</MenubarShortcut>
          </MenubarItem>
          <MenubarItem disabled>
            Redo
            <MenubarShortcut>Ctrl+Shift+Z</MenubarShortcut>
          </MenubarItem>
          <MenubarSeparator />
          <MenubarItem disabled>Preferences</MenubarItem>
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
