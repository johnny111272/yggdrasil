<script lang="ts">
  interface Props {
    icon?: string;
    message: string;
  }

  let { icon, message }: Props = $props();
</script>

<div class="empty-state">
  {#if icon}
    <span class="empty-icon">{icon}</span>
  {/if}
  <p class="empty-message">{message}</p>
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    flex: 1;
    padding: var(--space-4xl);
    color: var(--text-secondary);
  }

  .empty-icon {
    font-size: var(--text-3xl);
  }

  .empty-message {
    margin: 0;
    text-align: center;
  }
</style>
