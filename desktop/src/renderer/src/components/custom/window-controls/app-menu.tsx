import { Kbd, KbdGroup } from "@/components/ui/kbd";
import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
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
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>N</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarItem disabled>
            Open project
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>O</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarSeparator />
          <MenubarItem disabled>
            Save
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>S</Kbd>
            </KbdGroup>
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
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>Z</Kbd>
            </KbdGroup>
          </MenubarItem>
          <MenubarItem disabled>
            Redo
            <KbdGroup className="ml-auto">
              <Kbd>Ctrl</Kbd>
              <Kbd>Shift</Kbd>
              <Kbd>Z</Kbd>
            </KbdGroup>
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
