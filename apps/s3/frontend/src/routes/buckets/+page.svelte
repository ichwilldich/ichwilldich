<script lang="ts">
  import { Folder, Database, Plus } from '@lucide/svelte';
  import { Card, Button } from 'positron-components/components/ui';
  import { goto } from '$app/navigation';

  function handleNewBucket() {
    console.log('Create new bucket');
    // Hier würde eine Modal oder Navigation zu einer Create-Seite stattfinden
  }

  function handleViewAllBuckets() {
    console.log('View all buckets');
  }
</script>

<div class="flex h-full items-center justify-center">
  <div class="max-w-md space-y-6 p-8 text-center">
    <div class="bg-muted mx-auto w-fit rounded-full p-6">
      <Database class="text-muted-foreground h-16 w-16" />
    </div>
    <div class="space-y-3">
      <h2 class="text-2xl font-bold">S3 Object Storage</h2>
      <p class="text-muted-foreground">
        Verwalten Sie Ihre Buckets und Dateien in der Cloud. Wählen Sie ein
        Bucket aus der Seitenleiste aus oder erstellen Sie ein neues.
      </p>
    </div>
    <div class="space-y-3">
      <Button onclick={handleNewBucket} class="w-full">
        <Plus class="mr-2 h-4 w-4" />
        Neues Bucket erstellen
      </Button>
      <Button onclick={handleViewAllBuckets} variant="outline" class="w-full">
        <Folder class="mr-2 h-4 w-4" />
        Alle Buckets anzeigen
      </Button>
    </div>

    <Card.Root class="mt-6 text-left">
      <Card.Content class="p-4">
        <div class="space-y-2">
          <h4 class="text-sm font-semibold">Schnellstart</h4>
          <ul class="text-muted-foreground space-y-1 text-xs">
            <li>• Erstellen Sie Ihr erstes Bucket</li>
            <li>• Laden Sie Dateien hoch</li>
            <li>• Verwalten Sie Berechtigungen</li>
            <li>• Organisieren Sie Ihre Daten</li>
          </ul>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</div>
