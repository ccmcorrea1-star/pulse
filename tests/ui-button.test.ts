// @vitest-environment happy-dom

import { mount } from "@vue/test-utils";
import { afterEach, describe, expect, it } from "vitest";

import Button from "@/components/ui/button/Button.vue";

describe("Button", () => {
  const wrappers: ReturnType<typeof mount>[] = [];

  afterEach(() => {
    for (const wrapper of wrappers.splice(0)) {
      wrapper.unmount();
    }
  });

  it("renders its action label and semantic type", () => {
    const wrapper = mount(Button, { props: { type: "button" }, slots: { default: "Enviar fixture" } });
    wrappers.push(wrapper);

    expect(wrapper.get("button").text()).toBe("Enviar fixture");
    expect(wrapper.get("button").attributes("type")).toBe("button");
  });

  it("preserves disabled state as an observable interaction boundary", () => {
    const wrapper = mount(Button, { attrs: { disabled: true }, slots: { default: "Indisponível" } });
    wrappers.push(wrapper);

    expect(wrapper.get("button").attributes("disabled")).toBeDefined();
    expect(wrapper.text()).toContain("Indisponível");
  });
});
