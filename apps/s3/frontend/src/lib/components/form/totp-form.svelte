<script lang="ts">
  import { Card } from 'positron-components';
  import { Label } from 'positron-components';
  import { Input } from 'positron-components';
  import { Button } from 'positron-components';
  import { cn } from 'positron-components';
  import type { HTMLAttributes } from 'svelte/elements';
  let { class: className, ...restProps }: HTMLAttributes<HTMLDivElement> =
    $props();
  const id = $props.id();
  import { Database } from '@lucide/svelte';
</script>

<div class={cn('flex flex-col gap-6', className)} {...restProps}>
  <Card.Root class="overflow-hidden p-0">
    <!-- Image/Brand Seite zuerst, Formular danach (Seiten vertauscht) -->
    <Card.Content class="grid p-0 md:grid-cols-2">
      <div
        class="bg-muted relative hidden items-center justify-center p-8 md:flex"
      >
        <div class="flex flex-col items-center gap-3 text-center">
          <Database
            class="h-40 w-40 text-violet-500 dark:text-violet-400"
            aria-hidden="true"
          />
          <span
            class="text-lg font-semibold text-violet-600 dark:text-violet-300"
            >ichwilldich</span
          >
        </div>
      </div>

      <form class="p-6 md:p-8">
        <div class="flex flex-col gap-6">
          <div class="flex flex-col items-center text-center">
            <h1 class="text-2xl font-bold">Two‑Factor Authentication</h1>
            <p class="text-muted-foreground text-balance">
              Enter the 6‑digit code from your authenticator app
            </p>
          </div>

          <div class="grid gap-3">
            <Label for="totp-{id}">Authentication code</Label>
            <Input
              id="totp-{id}"
              type="text"
              inputmode="numeric"
              pattern="[0-9]*"
              maxlength="6"
              placeholder="123456"
              required
              class="text-center text-lg font-medium tracking-widest"
            />
          </div>

          <Button type="submit" class="w-full">Verify</Button>

          <div class="text-center text-sm">
            Lost access? <a href="##" class="underline underline-offset-4"
              >Use backup codes</a
            >
          </div>
        </div>
      </form>
    </Card.Content>
  </Card.Root>

  <div
    class="text-muted-foreground *:[a]:hover:text-primary text-center text-xs text-balance *:[a]:underline *:[a]:underline-offset-4"
  >
    By verifying, you confirm your identity.
  </div>
</div>
