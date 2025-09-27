<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, Folder, Plus, BarChart3 } from '@lucide/svelte';
  import { Card, Button, Input } from 'positron-components/components/ui';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';

  interface Props {
    children: import('svelte').Snippet;
  }

  let { children }: Props = $props();
  // Beispiel-Daten für Buckets
  let buckets = [
    {
      id: '1',
      name: 'my-documents',
      description: 'Personal documents and files',
      size: '2.3 GB',
      objectCount: 145,
      created: '2024-01-15',
      region: 'eu-central-1',
      public: false
    },
    {
      id: '2',
      name: 'website-assets',
      description: 'Static assets for website',
      size: '856 MB',
      objectCount: 67,
      created: '2024-02-08',
      region: 'us-east-1',
      public: true
    },
    {
      id: '3',
      name: 'backup-data',
      description: 'System backups and archives',
      size: '15.7 GB',
      objectCount: 892,
      created: '2023-12-03',
      region: 'eu-west-2',
      public: false
    },
    {
      id: '4',
      name: 'media-files',
      description: 'Images, videos and audio files',
      size: '8.2 GB',
      objectCount: 234,
      created: '2024-03-12',
      region: 'ap-southeast-1',
      public: false
    },
    {
      id: '5',
      name: 'logs-storage',
      description: 'Application logs and metrics',
      size: '1.1 GB',
      objectCount: 1823,
      created: '2024-01-28',
      region: 'eu-central-1',
      public: false
    }
  ];

  let searchTerm = $state('');
  let filteredBuckets = $state(buckets);

  // Aktive Bucket-ID aus der URL
  let selectedBucketId = $derived($page.params.id);
  let selectedBucket = $derived(buckets.find((b) => b.id === selectedBucketId));

  // Suchfunktion als Effect
  $effect(() => {
    if (searchTerm.trim() === '') {
      filteredBuckets = buckets;
    } else {
      filteredBuckets = buckets.filter((bucket) =>
        bucket.name.toLowerCase().includes(searchTerm.toLowerCase())
      );
    }
  });

  // Handler für Bucket-Klick
  function handleBucketClick(bucket: any) {
    goto(`/buckets/${bucket.id}`);
  }

  // Handler für neues Bucket
  function handleNewBucket() {
    console.log('Create new bucket');
    // Hier würdest du zu einer Create-Seite navigieren
    // goto('/buckets/new');
  }

  onMount(() => {
    console.log('Buckets layout loaded');
  });
</script>

<div class="flex h-screen flex-col">
  <div class="flex min-h-0 flex-1 gap-6 p-6">
    <!-- Sidebar Card -->
    <Card.Root class="flex w-80 flex-col">
      <Card.Header class="pb-4">
        <Card.Title class="flex items-center gap-2 text-lg font-semibold">
          <Folder class="text-primary h-5 w-5" />
          S3 Buckets
        </Card.Title>
      </Card.Header>

      <Card.Content class="flex min-h-0 flex-1 flex-col space-y-4 px-6 pb-6">
        <!-- Suchleiste -->
        <div class="relative">
          <Search
            class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform"
          />
          <Input
            type="text"
            placeholder="Buckets durchsuchen..."
            bind:value={searchTerm}
            class="pl-10"
          />
        </div>

        <!-- Dashboard Button -->
        <Button
          onclick={() => goto('/buckets/dashboard')}
          variant="outline"
          class="w-full"
        >
          <BarChart3 class="mr-2 h-4 w-4" />
          Dashboard
        </Button>

        <!-- Neuer Bucket Button -->
        <Button onclick={handleNewBucket} class="w-full">
          <Plus class="mr-2 h-4 w-4" />
          Neues Bucket
        </Button>

        <!-- Bucket-Liste -->
        <div class="-mx-2 min-h-0 flex-1 overflow-y-auto">
          {#if filteredBuckets.length === 0}
            <div class="p-4 text-center">
              <Folder class="text-muted-foreground mx-auto mb-2 h-8 w-8" />
              <p class="text-muted-foreground text-sm">
                Keine Buckets gefunden
              </p>
            </div>
          {:else}
            <div class="space-y-1 px-2">
              {#each filteredBuckets as bucket (bucket.id)}
                <Button
                  variant={selectedBucketId === bucket.id ? 'default' : 'ghost'}
                  class="h-auto w-full justify-start p-3"
                  onclick={() => handleBucketClick(bucket)}
                >
                  <div class="flex w-full items-center gap-3">
                    <Folder class="h-4 w-4 flex-shrink-0" />
                    <div class="flex min-w-0 flex-1 flex-col items-start">
                      <span
                        class="w-full truncate text-left text-sm font-medium"
                      >
                        {bucket.name}
                      </span>
                      <span
                        class="text-muted-foreground w-full truncate text-left text-xs"
                      >
                        {bucket.objectCount} Objekte
                      </span>
                    </div>
                  </div>
                </Button>
              {/each}
            </div>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>

    <!-- Main Content Card -->
    <Card.Root class="min-h-0 flex-1">
      <Card.Content class="h-full p-0">
        {@render children()}
      </Card.Content>
    </Card.Root>
  </div>
</div>

<style>
  /* Custom scrollbar styling */
  :global(.overflow-y-auto::-webkit-scrollbar) {
    width: 6px;
  }

  :global(.overflow-y-auto::-webkit-scrollbar-track) {
    background: hsl(var(--muted));
    border-radius: 3px;
  }

  :global(.overflow-y-auto::-webkit-scrollbar-thumb) {
    background: hsl(var(--muted-foreground) / 0.3);
    border-radius: 3px;
  }

  :global(.overflow-y-auto::-webkit-scrollbar-thumb:hover) {
    background: hsl(var(--muted-foreground) / 0.5);
  }
</style>
