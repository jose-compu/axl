export function ValidA11y() {
  return (
    <section>
      <div role="dialog" title="Container title">
        <button aria-label="Close" tabIndex={0}>
          Close
        </button>
      </div>
    </section>
  );
}
