<script lang="ts">
  interface Props {
    checked: boolean;
    onchange: (value: boolean) => void;
    label?: string;
  }

  let { checked, onchange, label = "" }: Props = $props();
</script>

<button
  class="switch"
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  onclick={(e) => {
    e.stopPropagation();
    onchange(!checked);
  }}
>
  <span class="knob"></span>
</button>

<style>
  .switch {
    width: 38px;
    height: 22px;
    border: none;
    border-radius: 13px;
    padding: 2px;
    display: flex;
    justify-content: flex-start;
    background: oklch(0.86 0.008 70);
    cursor: pointer;
    flex: none;
  }
  .switch[aria-checked="true"] {
    background: var(--accent);
    justify-content: flex-end;
  }
  .knob {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--card);
    box-shadow: 0 1px 3px oklch(0.24 0.012 60 / 0.3);
  }
</style>
