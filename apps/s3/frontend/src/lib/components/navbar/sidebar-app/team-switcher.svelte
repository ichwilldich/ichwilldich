<script lang="ts">
  import { DropdownMenu } from 'positron-components/components/ui';
  import { Sidebar } from 'positron-components/components/ui';
  import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import { Database } from '@lucide/svelte';
  // This should be `Component` after @lucide/svelte updates types
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let { teams }: { teams: { name: string; logo: any; plan: string }[] } =
    $props();
  let sidebar = Sidebar.useSidebar();
  let activeTeam = $state(teams[0]);
  const team = { name: 'ichwilldich', version: '0.0.1 beta', logo: Database };

  // Öffnet / toggled die Sidebar (verschiedene APIs abdecken)
  function openSidebar() {
    if (!sidebar) return;
    if (typeof sidebar.toggle === 'function') sidebar.toggle();
    else if (typeof sidebar.open === 'function') sidebar.open();
    else if (typeof sidebar.setOpen === 'function') sidebar.setOpen(true);
  }
</script>

<Sidebar.Menu>
  <Sidebar.MenuItem>
    <Sidebar.MenuButton
      size="lg"
      class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
      onclick={openSidebar}
    >
      <div
        class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
      >
        <team.logo class="size-4" />
      </div>
      <div class="grid flex-1 text-left text-sm leading-tight">
        <span class="truncate font-medium">
          {team.name}
        </span>
        <span class="truncate text-xs">{team.version}</span>
      </div>
    </Sidebar.MenuButton>
  </Sidebar.MenuItem>
</Sidebar.Menu>
