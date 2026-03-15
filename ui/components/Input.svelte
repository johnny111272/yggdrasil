<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    type?: "text" | "search" | "password";
    disabled?: boolean;
    oninput?: (e: Event) => void;
    onkeydown?: (e: KeyboardEvent) => void;
  }

  let { value = $bindable(""), placeholder = "", type = "text", disabled = false, oninput, onkeydown }: Props = $props();
</script>

<input
  class="input"
  {type}
  bind:value
  {placeholder}
  {disabled}
  {oninput}
  {onkeydown}
/>

<style>
  .input {
    width: 100%;
    padding: var(--space-md) var(--space-xl);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: var(--font-body);
    font-size: var(--text-sm);
  }

  .input::placeholder {
    color: var(--text-secondary);
  }

  .input:focus {
    outline: none;
    border-color: var(--action-primary);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
