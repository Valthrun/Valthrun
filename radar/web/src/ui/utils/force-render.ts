import { useCallback, useState } from "react";

export function useForceUpdate(): () => void {
    const [_state, setState] = useState<number>(0);
    return useCallback(() => {
        setState(performance.now());
    }, [setState]);
}