<script lang="ts">
  let { content }: { content: string } = $props();
</script>

<section class="markdown-preview">
  {@html content}
</section>

<style>
  .markdown-preview {
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    padding: var(--space-2xl);
    overflow-y: auto;
    line-height: 1.7;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3),
  .markdown-preview :global(h4) {
    color: var(--text-primary);
    margin-top: var(--space-2xl);
    margin-bottom: var(--space-lg);
  }

  .markdown-preview :global(h1) {
    font-size: var(--text-3xl);
    border-bottom: 1px solid var(--border-default);
    padding-bottom: var(--space-md);
  }

  .markdown-preview :global(h2) {
    font-size: var(--text-2xl);
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: var(--space-sm);
  }

  .markdown-preview :global(h3) {
    font-size: var(--text-xl);
  }

  .markdown-preview :global(p) {
    margin-bottom: var(--space-lg);
    color: var(--text-muted);
  }

  .markdown-preview :global(code) {
    background: var(--bg-primary);
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.9em;
  }

  .markdown-preview :global(pre) {
    background: var(--bg-primary);
    padding: var(--space-lg);
    border-radius: var(--radius-md);
    overflow-x: auto;
    margin-bottom: var(--space-lg);
  }

  .markdown-preview :global(pre code) {
    background: none;
    padding: 0;
  }

  .markdown-preview :global(ul),
  .markdown-preview :global(ol) {
    margin-bottom: var(--space-lg);
    padding-left: var(--space-2xl);
    color: var(--text-muted);
  }

  .markdown-preview :global(li) {
    margin-bottom: var(--space-sm);
  }

  .markdown-preview :global(blockquote) {
    border-left: 4px solid var(--action-primary);
    margin: var(--space-lg) 0;
    padding: var(--space-md) var(--space-xl);
    background: var(--bg-primary);
    color: var(--text-secondary);
  }

  .markdown-preview :global(a) {
    color: var(--action-primary);
    text-decoration: none;
  }

  .markdown-preview :global(a:hover) {
    text-decoration: underline;
  }

  .markdown-preview :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: var(--space-lg);
  }

  .markdown-preview :global(th),
  .markdown-preview :global(td) {
    border: 1px solid var(--border-default);
    padding: var(--space-md);
    text-align: left;
  }

  .markdown-preview :global(th) {
    background: var(--bg-primary);
    font-weight: bold;
  }

  .markdown-preview :global(hr) {
    border: none;
    border-top: 1px solid var(--border-default);
    margin: var(--space-2xl) 0;
  }
</style>
