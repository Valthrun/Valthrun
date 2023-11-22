import { Box, Button, Typography, CircularProgress, Input, TextField, Alert } from "@mui/material";
import * as React from "react";
import { SubscriberClientProvider, useSubscriberClient } from "../../../components/connection";
import { useParams } from "react-router-dom";
import { RadarState } from "../../../../backend/connection";
import { ContextRadarState, RadarRenderer } from "./radar";

export default React.memo(() => {
    return (
        <Box sx={{
            height: "100%",
            width: "100%",

            display: "flex",
            flexDirection: "column",
            justifyContent: "center"
        }}>
            <SubscriberClientProvider address={"ws://127.0.0.1:7229/subscribe"}>
                <ClientStateNew />
                <ClientStateConnecting />
                <ClientStateFailed />
                <ClientStateConnected />
            </SubscriberClientProvider>
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

const ClientStateConnected = React.memo(() => {
    const client = useSubscriberClient();
    const state = useSubscriberClientState();
    const [radarState, setRadarState] = React.useState<RadarState>({
        players: [],
        worldName: "de_anubis"
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