import { Box, Typography, CircularProgress, Alert, IconButton } from "@mui/material";
import * as React from "react";
import { SubscriberClientProvider, useSubscriberClient } from "../../../components/connection";
import { useParams } from "react-router-dom";
import { RadarState } from "../../../../backend/connection";
import { ContextRadarState, RadarRenderer } from "./radar";
import { useAppDispatch } from "../../../../state";
import ModalSettings from "./modal-settings";
import { Settings as IconSettings } from "@mui/icons-material";
import { updateRadarSettings } from "../../../../state/radar-settings";

const kServerUrl: string | null = process.env.SERVER_URL;
export default React.memo(() => {
    const dispatch = useAppDispatch();
    const targetUrl = React.useMemo(() => {
        if (typeof kServerUrl === "string") {
            return kServerUrl;
        }

        const parts = [];
        if (location.protocol === "https:") {
            parts.push("wss://");
        } else {
            parts.push("ws://");
        }
        parts.push(location.hostname);
        if (location.port) {
            parts.push(`:${location.port}`);
        }
        parts.push("/subscribe");

        return parts.join("");
    }, []);

    return (
        <Box sx={{
            height: "100%",
            width: "100%",

            display: "flex",
            flexDirection: "column",
            justifyContent: "center"
        }}>
            <SubscriberClientProvider address={targetUrl}>
                <ClientStateNew />
                <ClientStateConnecting />
                <ClientStateFailed />
                <ClientStateConnected />
                <ClientStateDisconnected />
            </SubscriberClientProvider>
            <ModalSettings />
            <Box sx={{ position: "absolute", top: 0, right: 0 }}>
                <IconButton onClick={() => dispatch(updateRadarSettings({ dialogOpen: true }))} sx={{ mr: 2, mt: 2 }}>
                    <IconSettings />
                </IconButton>
            </Box>
        </Box>
    );
});

const useSubscriberClientState = () => {
    const client = useSubscriberClient();
    const [state, setState] = React.useState(() => client.getState());
    React.useEffect(() => client.events.on("state_changed", newState => setState(newState)), [client]);
    return state;
}

const ClientStateNew = React.memo(() => {
    const client = useSubscriberClient();
    const { state } = useSubscriberClientState();
    const { sessionId } = useParams() as any;

    React.useEffect(() => {
        if (state !== "new") {
            return;
        }

        client.connect(sessionId);
    }, [client, state]);

    if (state !== "new") {
        return null;
    }

    if (!sessionId) {
        return (
            <Alert severity={"error"}>
                Missing session id in URL
            </Alert>
        );
    }

    return null;
});

const ClientStateConnecting = React.memo(() => {
    const { state } = useSubscriberClientState();
    if (state !== "connecting") {
        return;
    }

    return (
        <Box sx={{ alignSelf: "center" }}>
            <CircularProgress />
            <Typography>Connecting</Typography>
        </Box>
    );
});

const ClientStateFailed = React.memo(() => {
    const state = useSubscriberClientState();
    if (state.state !== "failed") {
        return;
    }

    return (
        <Box sx={{ alignSelf: "center" }}>
            <Typography>Connection Error</Typography>
            <Typography>{state.reason}</Typography>
        </Box>
    );
});

const ClientStateDisconnected = React.memo(() => {
    const state = useSubscriberClientState();
    if (state.state !== "disconnected") {
        return;
    }

    return (
        <Box sx={{ alignSelf: "center" }}>
            <Typography>Session has been closed</Typography>
        </Box>
    );
});

const ClientStateConnected = React.memo(() => {
    const client = useSubscriberClient();
    const state = useSubscriberClientState();
    const [radarState, setRadarState] = React.useState<RadarState>({
        players: [],
        worldName: "de_anubis",
        bomb: null,
    });

    React.useEffect(() => client.events.on("radar.state", update => setRadarState(update)), [client]);

    if (state.state !== "connected") {
        return;
    }

    return (
        <Box sx={{ alignSelf: "center", height: "100%", width: "100%", display: "flex", flexDirection: "column", justifyContent: "center" }}>
            <ContextRadarState.Provider value={radarState}>
                <RadarRenderer />
            </ContextRadarState.Provider>
        </Box>
    );
})