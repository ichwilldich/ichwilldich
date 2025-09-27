<script lang="ts">
  import {
    HardDrive,
    Calendar,
    Globe,
    Lock,
    Database,
    Folder,
    Upload,
    Download,
    File,
    Image,
    Video,
    FileText,
    Archive,
    Music,
    MoreVertical,
    Search,
    Grid,
    List,
    ArrowUp,
    Trash2,
    Edit,
    Copy,
    Share,
    Eye,
    RefreshCw,
    Plus
  } from '@lucide/svelte';
  import {
    Card,
    Badge,
    Button,
    Input,
    Separator,
    DropdownMenu,
    Tabs
  } from 'positron-components/components/ui';

  import { page } from '$app/stores';

  // Buckets-Daten (sollten aus einem Store oder API kommen)
  const buckets = [
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

  // Aktuelles Bucket aus URL-Parameter
  let selectedBucket = $derived(buckets.find((b) => b.id === $page.params.id));

  // Demo Files für das File System
  let files = $state([
    {
      id: '1',
      name: 'documents',
      type: 'folder',
      size: null,
      modified: '2024-09-15T10:30:00Z',
      objects: 24
    },
    {
      id: '2',
      name: 'images',
      type: 'folder',
      size: null,
      modified: '2024-09-18T14:45:00Z',
      objects: 156
    },
    {
      id: '3',
      name: 'presentation.pdf',
      type: 'file',
      extension: 'pdf',
      size: 2400000,
      modified: '2024-09-20T09:15:00Z'
    },
    {
      id: '4',
      name: 'logo.png',
      type: 'file',
      extension: 'png',
      size: 156000,
      modified: '2024-09-19T16:20:00Z'
    },
    {
      id: '5',
      name: 'backup.zip',
      type: 'file',
      extension: 'zip',
      size: 164000000,
      modified: '2024-09-17T11:30:00Z'
    },
    {
      id: '6',
      name: 'video-demo.mp4',
      type: 'file',
      extension: 'mp4',
      size: 45000000,
      modified: '2024-09-16T13:45:00Z'
    }
  ]);

  let viewMode = $state('grid');
  let searchTerm = $state('');
  let currentPath = $state<string[]>([]);
  let selectedFiles = $state(new Set());

  let filteredFiles = $derived(
    files.filter((file) =>
      file.name.toLowerCase().includes(searchTerm.toLowerCase())
    )
  );

  // Formatierung für Datum
  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString('de-DE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  // Formatierung für Dateigröße
  function formatFileSize(bytes: number | null) {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
  }

  // Icon basierend auf Dateityp
  function getFileIcon(file: any) {
    if (file.type === 'folder') return Folder;

    switch (file.extension) {
      case 'pdf':
      case 'doc':
      case 'docx':
        return FileText;
      case 'png':
      case 'jpg':
      case 'jpeg':
      case 'gif':
      case 'svg':
        return Image;
      case 'mp4':
      case 'avi':
      case 'mov':
        return Video;
      case 'mp3':
      case 'wav':
      case 'flac':
        return Music;
      case 'zip':
      case 'rar':
      case '7z':
        return Archive;
      default:
        return File;
    }
  }

  // Handler-Funktionen
  function handleFileClick(file: any) {
    if (file.type === 'folder') {
      currentPath = [...currentPath, file.name];
      console.log(`Navigating to folder: ${file.name}`);
    } else {
      console.log(`Opening file: ${file.name}`);
    }
  }

  function handleFileSelect(file: any) {
    if (selectedFiles.has(file.id)) {
      selectedFiles.delete(file.id);
    } else {
      selectedFiles.add(file.id);
    }
    selectedFiles = selectedFiles;
  }

  function handleUpload() {
    console.log('Upload files');
  }

  function handleDownload(file: any) {
    console.log(`Download file: ${file.name}`);
  }

  function handleDelete(file: any) {
    console.log(`Delete file: ${file.name}`);
  }

  function handleRefresh() {
    console.log('Refresh file list');
  }

  function navigateUp() {
    if (currentPath.length > 0) {
      currentPath = currentPath.slice(0, -1);
    }
  }
</script>

{#if selectedBucket}
  <div class="flex h-full flex-col">
    <!-- Header -->
    <div class="border-b p-6">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="bg-primary/10 rounded-lg p-2">
            <Folder class="text-primary h-6 w-6" />
          </div>
          <div>
            <h1 class="text-2xl font-bold">{selectedBucket.name}</h1>
            <p class="text-muted-foreground">{selectedBucket.description}</p>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <Badge variant={selectedBucket.public ? 'default' : 'secondary'}>
            {#if selectedBucket.public}
              <Globe class="mr-1 h-3 w-3" />
              Öffentlich
            {:else}
              <Lock class="mr-1 h-3 w-3" />
              Privat
            {/if}
          </Badge>
          <Button onclick={handleRefresh} variant="outline" size="sm">
            <RefreshCw class="h-4 w-4" />
          </Button>
        </div>
      </div>

      <!-- Stats Bar -->
      <div class="text-muted-foreground mt-4 flex items-center gap-6 text-sm">
        <div class="flex items-center gap-1">
          <HardDrive class="h-4 w-4" />
          <span>{selectedBucket.size}</span>
        </div>
        <div class="flex items-center gap-1">
          <Database class="h-4 w-4" />
          <span>{selectedBucket.objectCount.toLocaleString()} Objekte</span>
        </div>
        <div class="flex items-center gap-1">
          <Globe class="h-4 w-4" />
          <span>{selectedBucket.region}</span>
        </div>
        <div class="flex items-center gap-1">
          <Calendar class="h-4 w-4" />
          <span>Erstellt {formatDate(selectedBucket.created)}</span>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 p-6">
      <Tabs.Root value="files" class="flex h-full flex-col">
        <Tabs.List class="grid w-full grid-cols-3">
          <Tabs.Trigger value="files">Dateien</Tabs.Trigger>
          <Tabs.Trigger value="settings">Einstellungen</Tabs.Trigger>
          <Tabs.Trigger value="permissions">Berechtigungen</Tabs.Trigger>
        </Tabs.List>

        <!-- Files Tab -->
        <Tabs.Content value="files" class="mt-6 flex-1 space-y-4">
          <!-- File Manager Toolbar -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <!-- Breadcrumb -->
              <div class="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => (currentPath = [])}
                >
                  <Folder class="mr-1 h-4 w-4" />
                  {selectedBucket.name}
                </Button>
                {#each currentPath as pathSegment, index}
                  <span class="text-muted-foreground">/</span>
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={() =>
                      (currentPath = currentPath.slice(0, index + 1))}
                  >
                    {pathSegment}
                  </Button>
                {/each}
              </div>

              {#if currentPath.length > 0}
                <Button variant="outline" size="sm" onclick={navigateUp}>
                  <ArrowUp class="mr-1 h-4 w-4" />
                  Zurück
                </Button>
              {/if}
            </div>

            <div class="flex items-center gap-3">
              <!-- Search -->
              <div class="relative">
                <Search
                  class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform"
                />
                <Input
                  placeholder="Dateien durchsuchen..."
                  bind:value={searchTerm}
                  class="w-64 pl-10"
                />
              </div>

              <!-- View Mode Toggle -->
              <div class="flex rounded-lg border">
                <Button
                  variant={viewMode === 'grid' ? 'default' : 'ghost'}
                  size="sm"
                  onclick={() => (viewMode = 'grid')}
                  class="rounded-r-none"
                >
                  <Grid class="h-4 w-4" />
                </Button>
                <Button
                  variant={viewMode === 'list' ? 'default' : 'ghost'}
                  size="sm"
                  onclick={() => (viewMode = 'list')}
                  class="rounded-l-none"
                >
                  <List class="h-4 w-4" />
                </Button>
              </div>

              <!-- Upload Button -->
              <Button onclick={handleUpload}>
                <Upload class="mr-2 h-4 w-4" />
                Hochladen
              </Button>
            </div>
          </div>

          <!-- File List/Grid -->
          <Card.Root class="flex-1">
            <Card.Content class="p-6">
              {#if filteredFiles.length === 0}
                <div class="py-12 text-center">
                  <Folder
                    class="text-muted-foreground mx-auto mb-4 h-12 w-12"
                  />
                  <h3 class="text-lg font-medium">Keine Dateien gefunden</h3>
                  <p class="text-muted-foreground mb-4">
                    {searchTerm
                      ? 'Keine Dateien entsprechen Ihrer Suche.'
                      : 'Dieses Verzeichnis ist leer.'}
                  </p>
                  <Button onclick={handleUpload}>
                    <Upload class="mr-2 h-4 w-4" />
                    Erste Datei hochladen
                  </Button>
                </div>
              {:else if viewMode === 'grid'}
                <!-- Grid View -->
                <div
                  class="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6"
                >
                  {#each filteredFiles as file}
                    {@const IconComponent = getFileIcon(file)}
                    <div class="group relative">
                      <Card.Root
                        class="cursor-pointer border-2 transition-all duration-200 hover:shadow-md {selectedFiles.has(
                          file.id
                        )
                          ? 'border-primary'
                          : 'border-border'}"
                      >
                        <Card.Content class="p-4 text-center">
                          <div class="flex flex-col items-center gap-2">
                            <!-- File Icon -->
                            <div class="bg-muted rounded-lg p-3">
                              <IconComponent
                                class="text-muted-foreground h-8 w-8"
                              />
                            </div>

                            <!-- File Name -->
                            <div class="w-full">
                              <div
                                class="truncate text-sm font-medium"
                                title={file.name}
                              >
                                {file.name}
                              </div>
                              <div class="text-muted-foreground mt-1 text-xs">
                                {#if file.type === 'folder'}
                                  {file.objects} Objekte
                                {:else}
                                  {formatFileSize(file.size)}
                                {/if}
                              </div>
                            </div>
                          </div>
                        </Card.Content>
                      </Card.Root>

                      <!-- Context Menu Button -->
                      <div
                        class="absolute top-2 right-2 opacity-0 transition-opacity group-hover:opacity-100"
                      >
                        <DropdownMenu.Root>
                          <DropdownMenu.Trigger>
                            <Button
                              variant="ghost"
                              size="sm"
                              class="h-8 w-8 p-0"
                            >
                              <MoreVertical class="h-4 w-4" />
                            </Button>
                          </DropdownMenu.Trigger>
                          <DropdownMenu.Content>
                            {#if file.type === 'file'}
                              <DropdownMenu.Item
                                onclick={() => handleDownload(file)}
                              >
                                <Download class="mr-2 h-4 w-4" />
                                Herunterladen
                              </DropdownMenu.Item>
                              <DropdownMenu.Item>
                                <Eye class="mr-2 h-4 w-4" />
                                Vorschau
                              </DropdownMenu.Item>
                            {/if}
                            <DropdownMenu.Item>
                              <Copy class="mr-2 h-4 w-4" />
                              Link kopieren
                            </DropdownMenu.Item>
                            <DropdownMenu.Item>
                              <Edit class="mr-2 h-4 w-4" />
                              Umbenennen
                            </DropdownMenu.Item>
                            <DropdownMenu.Item>
                              <Share class="mr-2 h-4 w-4" />
                              Teilen
                            </DropdownMenu.Item>
                            <DropdownMenu.Separator />
                            <DropdownMenu.Item
                              onclick={() => handleDelete(file)}
                              class="text-destructive"
                            >
                              <Trash2 class="mr-2 h-4 w-4" />
                              Löschen
                            </DropdownMenu.Item>
                          </DropdownMenu.Content>
                        </DropdownMenu.Root>
                      </div>

                      <!-- Click Handler -->
                      <button
                        class="absolute inset-0 h-full w-full"
                        aria-label="{file.name} {file.type === 'folder'
                          ? 'Ordner öffnen'
                          : 'Datei auswählen'}"
                        onclick={() => handleFileClick(file)}
                        ondblclick={() => handleFileSelect(file)}
                      ></button>
                    </div>
                  {/each}
                </div>
              {:else}
                <!-- List View -->
                <div class="space-y-1">
                  <div
                    class="text-muted-foreground grid grid-cols-12 gap-4 border-b px-4 py-2 text-sm font-medium"
                  >
                    <div class="col-span-6">Name</div>
                    <div class="col-span-2">Größe</div>
                    <div class="col-span-3">Geändert</div>
                    <div class="col-span-1"></div>
                  </div>
                  {#each filteredFiles as file}
                    {@const IconComponent = getFileIcon(file)}
                    <div
                      class="hover:bg-muted/50 group grid cursor-pointer grid-cols-12 gap-4 rounded-lg px-4 py-3 {selectedFiles.has(
                        file.id
                      )
                        ? 'bg-primary/10'
                        : ''}"
                    >
                      <button
                        class="col-span-6 flex w-full items-center gap-3 border-0 bg-transparent p-0 text-left"
                        onclick={() => handleFileClick(file)}
                      >
                        <IconComponent
                          class="text-muted-foreground h-5 w-5 flex-shrink-0"
                        />
                        <span class="truncate font-medium">{file.name}</span>
                      </button>
                      <div
                        class="text-muted-foreground col-span-2 flex items-center text-sm"
                      >
                        {#if file.type === 'folder'}
                          {file.objects} Objekte
                        {:else}
                          {formatFileSize(file.size)}
                        {/if}
                      </div>
                      <div
                        class="text-muted-foreground col-span-3 flex items-center text-sm"
                      >
                        {formatDate(file.modified)}
                      </div>
                      <div class="col-span-1 flex items-center justify-end">
                        <div
                          class="opacity-0 transition-opacity group-hover:opacity-100"
                        >
                          <DropdownMenu.Root>
                            <DropdownMenu.Trigger>
                              <Button
                                variant="ghost"
                                size="sm"
                                class="h-8 w-8 p-0"
                              >
                                <MoreVertical class="h-4 w-4" />
                              </Button>
                            </DropdownMenu.Trigger>
                            <DropdownMenu.Content>
                              {#if file.type === 'file'}
                                <DropdownMenu.Item
                                  onclick={() => handleDownload(file)}
                                >
                                  <Download class="mr-2 h-4 w-4" />
                                  Herunterladen
                                </DropdownMenu.Item>
                                <DropdownMenu.Item>
                                  <Eye class="mr-2 h-4 w-4" />
                                  Vorschau
                                </DropdownMenu.Item>
                              {/if}
                              <DropdownMenu.Item>
                                <Copy class="mr-2 h-4 w-4" />
                                Link kopieren
                              </DropdownMenu.Item>
                              <DropdownMenu.Item>
                                <Edit class="mr-2 h-4 w-4" />
                                Umbenennen
                              </DropdownMenu.Item>
                              <DropdownMenu.Item>
                                <Share class="mr-2 h-4 w-4" />
                                Teilen
                              </DropdownMenu.Item>
                              <DropdownMenu.Separator />
                              <DropdownMenu.Item
                                onclick={() => handleDelete(file)}
                                class="text-destructive"
                              >
                                <Trash2 class="mr-2 h-4 w-4" />
                                Löschen
                              </DropdownMenu.Item>
                            </DropdownMenu.Content>
                          </DropdownMenu.Root>
                        </div>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </Card.Content>
          </Card.Root>
        </Tabs.Content>

        <!-- Settings Tab -->
        <Tabs.Content value="settings" class="mt-6">
          <Card.Root>
            <Card.Header>
              <Card.Title>Bucket-Einstellungen</Card.Title>
              <Card.Description
                >Verwalten Sie die Konfiguration Ihres Buckets</Card.Description
              >
            </Card.Header>
            <Card.Content class="space-y-6">
              <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                <div class="space-y-2">
                  <div class="text-muted-foreground text-sm font-medium">
                    Bucket-Name
                  </div>
                  <div class="bg-muted rounded-lg p-3">
                    <code class="text-sm">{selectedBucket.name}</code>
                  </div>
                </div>
                <div class="space-y-2">
                  <div class="text-muted-foreground text-sm font-medium">
                    Region
                  </div>
                  <div class="bg-muted rounded-lg p-3">
                    <code class="text-sm">{selectedBucket.region}</code>
                  </div>
                </div>
              </div>

              <Separator />

              <div class="space-y-4">
                <h3 class="text-lg font-semibold">Sichtbarkeit</h3>
                <div class="flex items-center justify-between">
                  <div>
                    <div class="font-medium">Öffentlicher Zugriff</div>
                    <div class="text-muted-foreground text-sm">
                      Ermöglicht anonymen Lesezugriff auf alle Objekte
                    </div>
                  </div>
                  <Badge
                    variant={selectedBucket.public ? 'default' : 'secondary'}
                  >
                    {selectedBucket.public ? 'Aktiviert' : 'Deaktiviert'}
                  </Badge>
                </div>
              </div>

              <Separator />

              <div class="space-y-2">
                <div class="text-muted-foreground text-sm font-medium">
                  Beschreibung
                </div>
                <p class="text-muted-foreground text-sm">
                  {selectedBucket.description}
                </p>
              </div>
            </Card.Content>
          </Card.Root>
        </Tabs.Content>

        <!-- Permissions Tab -->
        <Tabs.Content value="permissions" class="mt-6">
          <Card.Root>
            <Card.Header>
              <Card.Title>Zugriffsberechtigungen</Card.Title>
              <Card.Description
                >Verwalten Sie die Berechtigungen für dieses Bucket</Card.Description
              >
            </Card.Header>
            <Card.Content>
              <div class="space-y-4">
                <div
                  class="flex items-center justify-between rounded-lg border p-4"
                >
                  <div class="flex items-center gap-3">
                    <div class="bg-primary/10 rounded-lg p-2">
                      <Globe class="text-primary h-5 w-5" />
                    </div>
                    <div>
                      <div class="font-medium">Öffentlicher Lesezugriff</div>
                      <div class="text-muted-foreground text-sm">
                        Jeder kann Dateien in diesem Bucket lesen
                      </div>
                    </div>
                  </div>
                  <Badge
                    variant={selectedBucket.public ? 'default' : 'secondary'}
                  >
                    {selectedBucket.public ? 'Aktiviert' : 'Deaktiviert'}
                  </Badge>
                </div>

                <div
                  class="flex items-center justify-between rounded-lg border p-4"
                >
                  <div class="flex items-center gap-3">
                    <div class="rounded-lg bg-green-100 p-2">
                      <Lock class="h-5 w-5 text-green-600" />
                    </div>
                    <div>
                      <div class="font-medium">Verschlüsselung</div>
                      <div class="text-muted-foreground text-sm">
                        Server-seitige Verschlüsselung aktiviert
                      </div>
                    </div>
                  </div>
                  <Badge variant="default">Aktiviert</Badge>
                </div>
              </div>
            </Card.Content>
          </Card.Root>
        </Tabs.Content>
      </Tabs.Root>
    </div>
  </div>
{:else}
  <!-- Placeholder -->
  <div class="flex h-full items-center justify-center">
    <div class="space-y-4 p-8 text-center">
      <div class="bg-muted mx-auto w-fit rounded-full p-4">
        <Folder class="text-muted-foreground h-12 w-12" />
      </div>
      <div class="space-y-2">
        <h3 class="text-xl font-semibold">Wählen Sie ein Bucket aus</h3>
        <p class="text-muted-foreground max-w-md">
          Klicken Sie auf ein Bucket in der Seitenleiste, um detaillierte
          Informationen und das Dateisystem anzuzeigen
        </p>
      </div>
      <Button variant="outline">
        <Database class="mr-2 h-4 w-4" />
        Neues Bucket erstellen
      </Button>
    </div>
  </div>
{/if}
