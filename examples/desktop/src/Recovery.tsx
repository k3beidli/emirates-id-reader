import { Component, type ReactNode } from "react";

/** Preserve an actionable UI if a component fails, instead of a blank window. */
export class Recovery extends Component<
  { children: ReactNode },
  { failed: boolean }
> {
  state = { failed: false };
  static getDerivedStateFromError() {
    return { failed: true };
  }
  render() {
    if (this.state.failed)
      return (
        <main className="recovery">
          <h1>Let’s reopen the reader.</h1>
          <p>The page could not be displayed.</p>
          <button className="primary" onClick={() => location.reload()}>
            Reopen app
          </button>
        </main>
      );
    return this.props.children;
  }
}
