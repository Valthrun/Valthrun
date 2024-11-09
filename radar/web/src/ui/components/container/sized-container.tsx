import React from "react";
import { Box, SxProps, Theme } from "@mui/material";
import { useEffect, useMemo, useRef, useState } from "react";

export type ContainerSize = {
    width: number,
    height: number
};

export default (props: {
    sx?: SxProps<Theme>,
    children: (size: ContainerSize) => React.ReactNode
}) => {
    const [currentSize, setSize] = useState<ContainerSize>({ width: 1, height: 1 });

    const refContainer = useRef<HTMLDivElement>();
    const observer = useMemo(() => {
        return new ResizeObserver(events => {
            const event = events[events.length - 1];
            const { width, height } = event.contentRect;
            setSize({ width, height });
        });
    }, [setSize]);

    useEffect(() => {
        if (!refContainer.current) {
            return;
        }

        observer.observe(refContainer.current);
        return () => observer.disconnect();
    }, [refContainer, observer]);

    return (
        <Box
            ref={refContainer}
            sx={{
                position: "absolute",

                top: 0,
                left: 0,
                right: 0,
                bottom: 0,

                ...props.sx
            }}
        >
            {props.children(currentSize)}
        </Box>
    )
};