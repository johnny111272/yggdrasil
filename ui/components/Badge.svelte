<script lang="ts">
  interface Props {
    count?: number;
    text?: string;
    severity?: "blocked" | "error" | "warning" | "success" | "neutral";
  }

  let { count, text, severity = "neutral" }: Props = $props();

  function getColor(level: string): string {
    switch (level) {
      case "blocked": return "var(--severity-blocked)";
      case "error": return "var(--severity-error)";
      case "warning": return "var(--severity-warning)";
      case "success": return "var(--severity-success)";
      default: return "var(--action-primary)";
    }
  }
</script>

<span class="badge" style="background: {getColor(severity)}">
  {text ?? count}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.5rem;
    padding: 0.125rem var(--space-md);
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--text-primary);
  }
</style>
