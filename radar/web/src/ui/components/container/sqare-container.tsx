import { Box, SxProps, Theme } from "@mui/material";
import React, { useContext } from "react";


const SqareSizeContext = React.createContext<number>(1);
export default React.memo((props: {
    children: React.ReactNode,
    sqareSize: number,
    sx?: SxProps<Theme>,
}) => {
    return (
        <Box
            sx={props.sx}
            style={{
                width: `${props.sqareSize}px`,
                height: `${props.sqareSize}px`,
            } as any}
        >
            <SqareSizeContext.Provider value={props.sqareSize}>
                {props.children}
            </SqareSizeContext.Provider>
        </Box>
    )
});

export const useSqareSize = () => useContext(SqareSizeContext);