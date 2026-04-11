export function InvalidA11y() {
  return (
    <section>
      <div role="widget">x</div>
      <button aria-label=" " tabIndex={4}>
        Click
      </button>
    </section>
  );
}
