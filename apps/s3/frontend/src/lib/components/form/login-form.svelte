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
  import { CloudCheck, KeyRound, Database } from '@lucide/svelte';
</script>

<div class={cn('flex flex-col gap-6', className)} {...restProps}>
  <Card.Root class="overflow-hidden p-0">
    <Card.Content class="grid p-0 md:grid-cols-2">
      <form class="p-6 md:p-8">
        <div class="flex flex-col gap-6">
          <div class="flex flex-col items-center text-center">
            <h1 class="text-2xl font-bold">Welcome back</h1>
            <p class="text-muted-foreground text-balance">
              Login to your account
            </p>
          </div>
          <div class="grid gap-3">
            <Label for="email-{id}">Email</Label>
            <Input
              id="email-{id}"
              type="email"
              placeholder="m@example.com"
              required
            />
          </div>
          <div class="grid gap-3">
            <div class="flex items-center">
              <Label for="password">Password</Label>
              <a
                href="##"
                class="ml-auto text-sm underline-offset-2 hover:underline"
              >
                Forgot your password?
              </a>
            </div>
            <Input id="password-{id}" type="password" required />
          </div>
          <Button type="submit" class="w-full">Login</Button>
          <div
            class="after:border-border relative text-center text-sm after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t"
          >
            <span class="bg-card text-muted-foreground relative z-10 px-2">
              Or continue with
            </span>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <Button
              variant="outline"
              type="button"
              class="hover:bg-accent hover:text-accent-foreground hover:border-primary focus:ring-ring flex w-full cursor-pointer items-center justify-center gap-2 transition-colors duration-150 ease-in-out focus:ring-2 focus:outline-none"
              onclick={() => {
                /* TODO: Passkey flow */
              }}
              aria-label="Sign in with Passkey"
              title="Sign in with Passkey"
            >
              <KeyRound class="h-4 w-4" />
              <span class="text-sm">Passkey</span>
            </Button>
            <Button
              variant="outline"
              type="button"
              class="hover:bg-accent hover:text-accent-foreground hover:border-primary focus:ring-ring flex w-full cursor-pointer items-center justify-center gap-2 transition-colors duration-150 ease-in-out focus:ring-2 focus:outline-none"
              onclick={() => {
                /* TODO: Azure SSO */
              }}
              aria-label="Sign in with Azure SSO"
              title="Sign in with Azure SSO"
            >
              <CloudCheck class="h-4 w-4" />
              <span class="text-sm">Azure</span>
            </Button>
          </div>
        </div>
      </form>
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
    </Card.Content>
  </Card.Root>
</div>
