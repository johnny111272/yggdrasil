<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    oninput?: (e: Event) => void;
  }

  let { value = $bindable(""), placeholder = "Search...", oninput }: Props = $props();
</script>

<div class="search-wrapper">
  <input
    class="search-input"
    type="search"
    bind:value
    {placeholder}
    {oninput}
  />
</div>

<style>
  .search-input {
    width: 100%;
    padding: var(--space-lg) var(--space-xl);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    box-sizing: border-box;
  }

  .search-input::placeholder {
    color: var(--text-secondary);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--action-primary);
  }
</style>
