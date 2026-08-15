import { useMediaQuery } from "@vueuse/core";

export function useResponsive() {
  const isCompact = useMediaQuery("(max-width: 920px)");
  const isMobile = useMediaQuery("(max-width: 680px)");

  return { isCompact, isMobile };
}
