export function ValidNaming() {
  return (
    <section>
      <span id="save-lbl">Save file</span>
      <button aria-labelledby="save-lbl" />
      <button aria-label="Close">X</button>
      <img src="/x.png" alt="Logo" />
      <button>Submit</button>
      <a href="/go">Home</a>
      <svg aria-label="Chart">
        <title>Chart</title>
      </svg>
      <label htmlFor="email">Email</label>
      <input id="email" />
    </section>
  );
}
