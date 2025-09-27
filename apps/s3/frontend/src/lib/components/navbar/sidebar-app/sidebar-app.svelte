<script lang="ts" module>
  import { Gauge, DatabaseIcon } from '@lucide/svelte';
  // This is sample data.
  const data = {
    user: {
      name: 'shadcn',
      email: 'm@example.com',
      avatar: '/avatars/shadcn.jpg'
    },
    teams: [],
    navMain: [
      {
        id: 'dashboard',
        title: 'Dashboard',
        url: '/',
        icon: Gauge
      },
      {
        id: 'buckets',
        title: 'Buckets',
        url: '/buckets',
        icon: DatabaseIcon
      }
    ]
  };
</script>

<script lang="ts">
  import NavMain from './nav-main.svelte';
  import NavUser from './nav-user.svelte';
  import TeamSwitcher from './team-switcher.svelte';
  import { Sidebar } from 'positron-components/components/ui';
  import type { ComponentProps } from 'svelte';
  let {
    ref = $bindable(null),
    collapsible = 'icon',
    ...restProps
  }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root {collapsible} {...restProps}>
  <Sidebar.Header>
    <TeamSwitcher teams={data.teams} />
  </Sidebar.Header>
  <Sidebar.Content>
    <NavMain items={data.navMain} />
  </Sidebar.Content>
  <Sidebar.Footer>
    <NavUser user={data.user} />
  </Sidebar.Footer>
  <Sidebar.Rail />
</Sidebar.Root>
