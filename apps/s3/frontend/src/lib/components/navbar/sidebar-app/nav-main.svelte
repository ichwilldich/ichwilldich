<script lang="ts">
  import { Collapsible } from 'positron-components/components/ui';
  import { Sidebar } from 'positron-components/components/ui';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import { Button } from 'bits-ui';
  let {
    items
  }: {
    items: {
      title: string;
      url: string;
      // this should be `Component` after @lucide/svelte updates types
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      icon?: any;
      isActive?: boolean;
    }[];
  } = $props();
</script>

<Sidebar.Group>
  <Sidebar.GroupLabel>Platform</Sidebar.GroupLabel>
  <Sidebar.Menu>
    {#each items as item (item.title)}
      <Sidebar.MenuItem>
        <a href={item.url}>
          <Sidebar.MenuButton tooltipContent={item.title}>
            {#if item.icon}
              <item.icon />
            {/if}
            <span>{item.title}</span>
          </Sidebar.MenuButton>
        </a>
      </Sidebar.MenuItem>
    {/each}
  </Sidebar.Menu>
</Sidebar.Group>
