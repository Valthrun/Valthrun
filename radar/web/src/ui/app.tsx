import * as React from "react";
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
import "./app.scss";
import { Box, CssBaseline, createTheme } from "@mui/material";
import { ThemeProvider } from "@emotion/react";
import { RecoilRoot } from "recoil";
import {
    Route,
    Link,
    BrowserRouter,
    Routes,
    Navigate
} from "react-router-dom";
import { Provider as StateProvider } from "react-redux";

import PageMain from "./pages/main";
import PageSession from "./pages/session/[id]";
import { appStore } from "../state";

const theme = createTheme({
    palette: {
        mode: 'dark',
    },
});

export const App = React.memo(() => {
    return (
        <React.Fragment>
            <StateProvider store={appStore}>
                <RecoilRoot>
                    <ThemeProvider theme={theme}>
                        <CssBaseline />
                        <BrowserRouter>
                            <Box sx={{
                                height: "100%",
                                width: "100%"
                            }}>
                                <Routes>
                                    <Route path="/" element={<PageMain />} />
                                    <Route path="/session/:sessionId" element={<PageSession />} />
                                    <Route path={"*"} element={<Navigate to={"/"} />} />
                                </Routes>
                            </Box>
                        </BrowserRouter>
                    </ThemeProvider>
                </RecoilRoot>
            </StateProvider>
        </React.Fragment>
    );
});