<script lang="ts">
  import TOTPForm from '$lib/components/form/totp-form.svelte';
</script>

<div
  class="bg-muted flex min-h-svh flex-col items-center justify-center p-6 md:p-10"
>
  <div class="w-full max-w-sm md:max-w-3xl">
    <TOTPForm />
  </div>
</div>
