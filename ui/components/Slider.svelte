<script lang="ts">
  interface Props {
    value: number;
    min?: number;
    max?: number;
    step?: number;
    oninput?: (e: Event) => void;
  }

  let { value = $bindable(0), min = 0, max = 100, step = 1, oninput }: Props = $props();
</script>

<input
  type="range"
  class="slider"
  bind:value
  {min}
  {max}
  {step}
  {oninput}
/>

<style>
  .slider {
    width: 100%;
    accent-color: var(--action-primary);
    cursor: pointer;
    height: 4px;
  }
</style>
