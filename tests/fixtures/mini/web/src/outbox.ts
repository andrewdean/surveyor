import { HttpClient, Handler } from "./client";
import React from "react";

/** Shows outbox depth in the admin UI. */
export class OutboxPanel extends HttpClient implements Handler {
  handle(): void {
    this.refresh();
  }

  refresh(): void {
    this.get("/api/outbox/depth");
  }
}

describe("OutboxPanel", () => {
  it("refreshes", () => {
    new OutboxPanel().handle();
  });
});
