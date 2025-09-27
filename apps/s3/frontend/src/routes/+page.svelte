<script lang="ts">
  import TrendingUpIcon from '@lucide/svelte/icons/trending-up';
  import { HardDrive, FileText, Server, Activity } from '@lucide/svelte/icons';
  import { Chart } from 'positron-components/components/ui';
  import { Card } from 'positron-components/components/ui';
  import { Select } from 'positron-components/components/ui';
  import { PieChart, Text } from 'layerchart';
  import { scaleUtc } from 'd3-scale';
  import { Area, AreaChart, ChartClipPath } from 'layerchart';
  import { curveNatural } from 'd3-shape';
  import { cubicInOut } from 'svelte/easing';

  // JSON Daten
  const dashboardData = {
    storage: [
      { type: 'used', size: 127.3, color: 'var(--color-used)' },
      { type: 'available', size: 372.7, color: 'var(--color-available)' }
    ],
    fileTypes: [
      { type: 'images', count: 1243, color: 'var(--color-images)' },
      { type: 'documents', count: 856, color: 'var(--color-documents)' },
      { type: 'videos', count: 432, color: 'var(--color-videos)' },
      { type: 'archives', count: 289, color: 'var(--color-archives)' },
      { type: 'other', count: 130, color: 'var(--color-other)' }
    ],
    activity: [
      { date: new Date('2024-09-01'), uploads: 222, downloads: 150 },
      { date: new Date('2024-09-02'), uploads: 97, downloads: 180 },
      { date: new Date('2024-09-03'), uploads: 167, downloads: 120 },
      { date: new Date('2024-09-04'), uploads: 242, downloads: 260 },
      { date: new Date('2024-09-05'), uploads: 373, downloads: 290 },
      { date: new Date('2024-09-06'), uploads: 301, downloads: 340 },
      { date: new Date('2024-09-07'), uploads: 245, downloads: 180 },
      { date: new Date('2024-09-08'), uploads: 409, downloads: 320 },
      { date: new Date('2024-09-09'), uploads: 59, downloads: 110 },
      { date: new Date('2024-09-10'), uploads: 261, downloads: 190 },
      { date: new Date('2024-09-11'), uploads: 327, downloads: 350 },
      { date: new Date('2024-09-12'), uploads: 292, downloads: 210 },
      { date: new Date('2024-09-13'), uploads: 342, downloads: 380 },
      { date: new Date('2024-09-14'), uploads: 137, downloads: 220 },
      { date: new Date('2024-09-15'), uploads: 120, downloads: 170 },
      { date: new Date('2024-09-16'), uploads: 138, downloads: 190 },
      { date: new Date('2024-09-17'), uploads: 446, downloads: 360 },
      { date: new Date('2024-09-18'), uploads: 364, downloads: 410 },
      { date: new Date('2024-09-19'), uploads: 243, downloads: 180 },
      { date: new Date('2024-09-20'), uploads: 89, downloads: 150 },
      { date: new Date('2024-09-21'), uploads: 137, downloads: 200 },
      { date: new Date('2024-09-22'), uploads: 224, downloads: 170 },
      { date: new Date('2024-09-23'), uploads: 138, downloads: 230 }
    ]
  };

  // Speicherverbrauch
  const storageData = dashboardData.storage;
  const totalStorage = storageData.reduce((acc, curr) => acc + curr.size, 0);
  const usedPercentage = ((storageData[0].size / totalStorage) * 100).toFixed(
    1
  );

  const storageConfig = {
    size: { label: 'Storage (GB)' },
    used: { label: 'Belegt', color: 'var(--chart-1)' },
    available: { label: 'Verfügbar', color: 'var(--chart-2)' }
  } satisfies Chart.ChartConfig;

  // Dateitypen
  const fileTypeData = dashboardData.fileTypes;
  const totalFiles = fileTypeData.reduce((acc, curr) => acc + curr.count, 0);

  const fileTypeConfig = {
    count: { label: 'Anzahl Dateien' },
    images: { label: 'Bilder', color: 'var(--chart-1)' },
    documents: { label: 'Dokumente', color: 'var(--chart-2)' },
    videos: { label: 'Videos', color: 'var(--chart-3)' },
    archives: { label: 'Archive', color: 'var(--chart-4)' },
    other: { label: 'Andere', color: 'var(--chart-5)' }
  } satisfies Chart.ChartConfig;

  // Aktivität Area Chart
  const activityData = dashboardData.activity;
  let timeRange = $state('30d');

  const selectedLabel = $derived.by(() => {
    switch (timeRange) {
      case '30d':
        return 'Letzte 30 Tage';
      case '14d':
        return 'Letzte 14 Tage';
      case '7d':
        return 'Letzte 7 Tage';
      default:
        return 'Letzte 30 Tage';
    }
  });

  const filteredActivityData = $derived(
    activityData.filter((item) => {
      const referenceDate = new Date('2024-09-23');
      let daysToSubtract = 30;
      if (timeRange === '14d') {
        daysToSubtract = 14;
      } else if (timeRange === '7d') {
        daysToSubtract = 7;
      }

      const cutoffDate = new Date(referenceDate);
      cutoffDate.setDate(cutoffDate.getDate() - daysToSubtract);
      return item.date >= cutoffDate;
    })
  );

  const activityConfig = {
    uploads: { label: 'Uploads', color: 'var(--chart-1)' },
    downloads: { label: 'Downloads', color: 'var(--chart-2)' }
  } satisfies Chart.ChartConfig;
</script>

<div class="space-y-6 p-6">
  <!-- Header -->
  <div class="flex flex-col gap-2">
    <h1 class="text-3xl font-bold tracking-tight">Dashboard</h1>
    <p class="text-muted-foreground">
      Übersicht über Speicherverbrauch, Dateitypen und Aktivität in Ihren S3
      Buckets
    </p>
  </div>

  <!-- KPI Cards -->
  <div class="grid gap-4 md:grid-cols-4">
    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Gesamt Speicher</Card.Title>
        <HardDrive class="text-muted-foreground h-4 w-4" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">{totalStorage.toFixed(1)} GB</div>
        <p class="text-muted-foreground text-xs">
          {usedPercentage}% belegt
        </p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Gesamt Dateien</Card.Title>
        <FileText class="text-muted-foreground h-4 w-4" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">{totalFiles.toLocaleString()}</div>
        <p class="text-muted-foreground text-xs">Über alle Buckets</p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Verfügbarer Speicher</Card.Title
        >
        <Server class="text-muted-foreground h-4 w-4" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">
          {storageData[1].size.toFixed(1)} GB
        </div>
        <p class="text-muted-foreground text-xs">
          {(100 - parseFloat(usedPercentage)).toFixed(1)}% frei
        </p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Tägliche Aktivität</Card.Title>
        <Activity class="text-muted-foreground h-4 w-4" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">
          {filteredActivityData.length > 0
            ? (
                filteredActivityData[filteredActivityData.length - 1].uploads +
                filteredActivityData[filteredActivityData.length - 1].downloads
              ).toLocaleString()
            : '0'}
        </div>
        <p class="text-muted-foreground text-xs">Uploads + Downloads heute</p>
      </Card.Content>
    </Card.Root>
  </div>

  <!-- Charts Grid -->
  <div class="grid gap-6 md:grid-cols-2">
    <!-- Speicherverbrauch Chart -->
    <Card.Root class="flex flex-col">
      <Card.Header class="items-center">
        <Card.Title>Speicherverbrauch</Card.Title>
        <Card.Description
          >Belegter vs. verfügbarer Speicherplatz</Card.Description
        >
      </Card.Header>
      <Card.Content class="flex-1">
        <Chart.Container
          config={storageConfig}
          class="mx-auto aspect-square max-h-[300px]"
        >
          <PieChart
            data={storageData}
            key="type"
            value="size"
            c="color"
            innerRadius={60}
            padding={28}
            props={{ pie: { motion: 'tween' } }}
          >
            {#snippet aboveMarks()}
              <Text
                value={`${storageData[0].size.toFixed(1)} GB`}
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-foreground text-2xl! font-bold"
                dy={-5}
              />
              <Text
                value="Belegt"
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-muted-foreground! text-muted-foreground text-sm!"
                dy={15}
              />
              <Text
                value={`${usedPercentage}%`}
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-muted-foreground! text-muted-foreground text-xs!"
                dy={30}
              />
            {/snippet}
            {#snippet tooltip()}
              <Chart.Tooltip hideLabel />
            {/snippet}
          </PieChart>
        </Chart.Container>
      </Card.Content>
      <Card.Footer class="flex-col gap-2 text-sm">
        <div class="flex items-center gap-2 leading-none font-medium">
          Speicherverbrauch im normalen Bereich <HardDrive class="size-4" />
        </div>
        <div class="text-muted-foreground leading-none">
          {storageData[1].size.toFixed(1)} GB von {totalStorage.toFixed(1)} GB verfügbar
        </div>
      </Card.Footer>
    </Card.Root>

    <!-- Dateitypen Chart -->
    <Card.Root class="flex flex-col">
      <Card.Header class="items-center">
        <Card.Title>Dateitypen Verteilung</Card.Title>
        <Card.Description>Aufschlüsselung nach Dateitypen</Card.Description>
      </Card.Header>
      <Card.Content class="flex-1">
        <Chart.Container
          config={fileTypeConfig}
          class="mx-auto aspect-square max-h-[300px]"
        >
          <PieChart
            data={fileTypeData}
            key="type"
            value="count"
            c="color"
            innerRadius={60}
            padding={28}
            props={{ pie: { motion: 'tween' } }}
          >
            {#snippet aboveMarks()}
              <Text
                value={String(totalFiles.toLocaleString())}
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-foreground text-2xl! font-bold"
                dy={-5}
              />
              <Text
                value="Dateien"
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-muted-foreground! text-muted-foreground text-sm!"
                dy={15}
              />
              <Text
                value="Gesamt"
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-muted-foreground! text-muted-foreground text-xs!"
                dy={30}
              />
            {/snippet}
            {#snippet tooltip()}
              <Chart.Tooltip hideLabel />
            {/snippet}
          </PieChart>
        </Chart.Container>
      </Card.Content>
      <Card.Footer class="flex-col gap-2 text-sm">
        <div class="flex items-center gap-2 leading-none font-medium">
          Bilder dominieren mit {(
            (fileTypeData[0].count / totalFiles) *
            100
          ).toFixed(1)}% <TrendingUpIcon class="size-4" />
        </div>
        <div class="text-muted-foreground leading-none">
          {fileTypeData[0].count.toLocaleString()} Bilder von {totalFiles.toLocaleString()}
          Dateien
        </div>
      </Card.Footer>
    </Card.Root>
  </div>

  <!-- Activity Area Chart - Full Width -->
  <Card.Root>
    <Card.Header
      class="flex items-center gap-2 space-y-0 border-b py-5 sm:flex-row"
    >
      <div class="grid flex-1 gap-1 text-center sm:text-left">
        <Card.Title>Aktivitätsverlauf</Card.Title>
        <Card.Description
          >Upload- und Download-Aktivität über Zeit</Card.Description
        >
      </div>
      <Select.Root type="single" bind:value={timeRange}>
        <Select.Trigger
          class="w-[160px] rounded-lg sm:ml-auto"
          aria-label="Zeitraum auswählen"
        >
          {selectedLabel}
        </Select.Trigger>
        <Select.Content class="rounded-xl">
          <Select.Item value="30d" class="rounded-lg"
            >Letzte 30 Tage</Select.Item
          >
          <Select.Item value="14d" class="rounded-lg"
            >Letzte 14 Tage</Select.Item
          >
          <Select.Item value="7d" class="rounded-lg">Letzte 7 Tage</Select.Item>
        </Select.Content>
      </Select.Root>
    </Card.Header>
    <Card.Content>
      <Chart.Container
        config={activityConfig}
        class="aspect-auto h-[300px] w-full"
      >
        <AreaChart
          legend
          data={filteredActivityData}
          x="date"
          xScale={scaleUtc()}
          series={[
            {
              key: 'downloads',
              label: 'Downloads',
              color: activityConfig.downloads.color
            },
            {
              key: 'uploads',
              label: 'Uploads',
              color: activityConfig.uploads.color
            }
          ]}
          seriesLayout="stack"
          props={{
            area: {
              curve: curveNatural,
              'fill-opacity': 0.4,
              line: { class: 'stroke-1' },
              motion: 'tween'
            },
            xAxis: {
              ticks: timeRange === '7d' ? 7 : undefined,
              format: (v) => {
                return v.toLocaleDateString('de-DE', {
                  month: 'short',
                  day: 'numeric'
                });
              }
            },
            yAxis: { format: () => '' }
          }}
        >
          {#snippet marks({ series, getAreaProps })}
            <defs>
              <linearGradient id="fillUploads" x1="0" y1="0" x2="0" y2="1">
                <stop
                  offset="5%"
                  stop-color="var(--color-uploads)"
                  stop-opacity={1.0}
                />
                <stop
                  offset="95%"
                  stop-color="var(--color-uploads)"
                  stop-opacity={0.1}
                />
              </linearGradient>
              <linearGradient id="fillDownloads" x1="0" y1="0" x2="0" y2="1">
                <stop
                  offset="5%"
                  stop-color="var(--color-downloads)"
                  stop-opacity={0.8}
                />
                <stop
                  offset="95%"
                  stop-color="var(--color-downloads)"
                  stop-opacity={0.1}
                />
              </linearGradient>
            </defs>
            <ChartClipPath
              initialWidth={0}
              motion={{
                width: { type: 'tween', duration: 1000, easing: cubicInOut }
              }}
            >
              {#each series as s, i (s.key)}
                <Area
                  {...getAreaProps(s, i)}
                  fill={s.key === 'uploads'
                    ? 'url(#fillUploads)'
                    : 'url(#fillDownloads)'}
                />
              {/each}
            </ChartClipPath>
          {/snippet}
          {#snippet tooltip()}
            <Chart.Tooltip
              labelFormatter={(v: Date) => {
                return v.toLocaleDateString('de-DE', {
                  weekday: 'long',
                  day: 'numeric',
                  month: 'long'
                });
              }}
              indicator="line"
            />
          {/snippet}
        </AreaChart>
      </Chart.Container>
    </Card.Content>
    <Card.Footer>
      <div class="flex w-full items-start gap-2 text-sm">
        <div class="grid gap-2">
          <div class="flex items-center gap-2 leading-none font-medium">
            Aktivität steigt um 8.2% in diesem Zeitraum <TrendingUpIcon
              class="size-4"
            />
          </div>
          <div
            class="text-muted-foreground flex items-center gap-2 leading-none"
          >
            September 2024 - Upload/Download Statistiken
          </div>
        </div>
      </div>
    </Card.Footer>
  </Card.Root>
</div>

<style>
  :root {
    --color-used: var(--chart-1);
    --color-available: var(--chart-2);
    --color-images: var(--chart-1);
    --color-documents: var(--chart-2);
    --color-videos: var(--chart-3);
    --color-archives: var(--chart-4);
    --color-other: var(--chart-5);
    --color-uploads: var(--chart-1);
    --color-downloads: var(--chart-2);
  }
</style>
