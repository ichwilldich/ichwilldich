<script lang="ts">
  import { ModeWatcher, Toaster } from 'positron-components/components/ui';
  import '../app.css';
  import AppSidebar from '$lib/components/navbar/sidebar-app/sidebar-app.svelte';
  import { Breadcrumb } from 'positron-components/components/ui';
  import { Separator } from 'positron-components/components/ui';
  import { Sidebar } from 'positron-components/components/ui';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';

  interface Props {
    children?: import('svelte').Snippet;
  }

  let { children }: Props = $props();

  const noLayout = ['/login', '/oauth', '/oauth/logout'];
</script>

<svelte:head></svelte:head>

{#if !noLayout.includes(page.url.pathname)}
  <Sidebar.Provider class="min-h-screen">
    <AppSidebar />
    <Sidebar.Trigger class="absolute top-3 left-3 flex md:hidden" />
    <main class="min-h-screen min-w-0 flex-1">
      <div class="w-full">
        {@render children?.()}
      </div>
    </main>
  </Sidebar.Provider>
{:else}
  {@render children?.()}
{/if}
