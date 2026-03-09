<script lang="ts">
  interface Props {
    value: number | string;
    label: string;
    severity?: "blocked" | "error" | "warning" | "success" | "neutral";
  }

  let { value, label, severity = "neutral" }: Props = $props();

  function getColor(sev: string): string {
    switch (sev) {
      case "blocked": return "var(--severity-blocked)";
      case "error": return "var(--severity-error)";
      case "warning": return "var(--severity-warning)";
      case "success": return "var(--severity-success)";
      default: return "var(--text-primary)";
    }
  }
</script>

<div class="stat-card">
  <span class="stat-value" style="color: {getColor(severity)}">{value}</span>
  <span class="stat-label">{label}</span>
</div>

<style>
  .stat-card {
    flex: 1;
    background: var(--bg-secondary);
    padding: var(--space-xl);
    border-radius: var(--radius-md);
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: var(--text-2xl);
    font-weight: bold;
  }

  .stat-label {
    color: var(--text-secondary);
    font-size: var(--text-sm);
  }
</style>
