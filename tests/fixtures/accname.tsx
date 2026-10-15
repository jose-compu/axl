export function AccNameExamples() {
  return (
    <section>
      <span id="by-id">Name from labelledby</span>
      <button aria-labelledby="by-id" aria-label="ignored">
        ignored
      </button>
      <button aria-label="Name from aria-label">ignored</button>
      <img src="/logo.png" alt="Name from alt" />
      <button>Name from button text</button>
    </section>
  );
}
