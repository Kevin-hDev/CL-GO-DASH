// Locks the ECharts roam assumptions the full-extent wheel guard relies on:
// wheel/roam-originated datazoom events carry `batch` (ours do not), and
// wheel zoom-out at [0,100] is a strict no-op upstream.
import { describe, expect, it } from "vitest";
import * as echarts from "echarts";

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function buildOption(start: number, end: number) {
  return {
    animation: false,
    grid: { left: 18, right: 18, top: 18, bottom: 38, containLabel: true },
    xAxis: { type: "time" },
    yAxis: { type: "value" },
    dataZoom: [
      {
        type: "inside",
        xAxisIndex: 0,
        start,
        end,
        minSpan: 10,
        realtime: true,
        throttle: 16,
        filterMode: "none",
        zoomOnMouseWheel: true,
        moveOnMouseWheel: false,
        moveOnMouseMove: false,
      },
    ],
    series: [
      {
        type: "line",
        data: Array.from({ length: 100 }, (_, i) => [
          Date.UTC(2020, 0, 1) + i * 86400000,
          i,
        ]),
      },
    ],
  };
}

function readZoom(chart: echarts.ECharts) {
  const zoom = (chart.getOption().dataZoom as { start: number; end: number }[])[0];
  return { start: zoom.start, end: zoom.end };
}

describe("echarts roam wheel assumptions", () => {
  it("marque les evenements molette avec batch, pas les dispatch manuels", async () => {
    const dom = document.createElement("div");
    document.body.appendChild(dom);
    const chart = echarts.init(dom, undefined, {
      renderer: "svg",
      width: 800,
      height: 400,
    });
    chart.setOption(buildOption(0, 100));
    const batches: boolean[] = [];
    chart.on("datazoom", (event: unknown) => {
      batches.push(Array.isArray((event as { batch?: unknown }).batch));
    });
    const target = (dom.firstElementChild ?? dom) as Element;
    target.dispatchEvent(
      new WheelEvent("wheel", {
        deltaY: -120,
        clientX: 400,
        clientY: 150,
        bubbles: true,
        cancelable: true,
      }),
    );
    await wait(60);
    chart.dispatchAction({ type: "dataZoom", start: 30, end: 70 });
    await wait(60);
    chart.dispose();
    dom.remove();
    expect(batches.length).toBeGreaterThanOrEqual(2);
    expect(batches[0]).toBe(true);
    expect(batches[1]).toBe(false);
  });

  it("ne bouge pas la fenetre au dezoom molette en pleine etendue", async () => {
    const dom = document.createElement("div");
    document.body.appendChild(dom);
    const chart = echarts.init(dom, undefined, {
      renderer: "svg",
      width: 800,
      height: 400,
    });
    chart.setOption(buildOption(0, 100));
    const target = (dom.firstElementChild ?? dom) as Element;
    for (const x of [200, 400, 600]) {
      target.dispatchEvent(
        new WheelEvent("wheel", {
          deltaY: 120,
          clientX: x,
          clientY: 150,
          bubbles: true,
          cancelable: true,
        }),
      );
      await wait(50);
    }
    const zoom = readZoom(chart);
    chart.dispose();
    dom.remove();
    expect(zoom).toEqual({ start: 0, end: 100 });
  });
});
