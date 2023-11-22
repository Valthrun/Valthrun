import { Box, Typography, TextField, Button } from "@mui/material";
import * as React from "react";
import { useNavigate } from "react-router-dom";

export default React.memo(() => {
    const navigate = useNavigate();
    const [sessionId, setSessionId] = React.useState("");

    return (
        <Box sx={{
            height: "100%",
            width: "100%",

            display: "flex",
            flexDirection: "column",
            justifyContent: "center"
        }}>
            <Box sx={{ alignSelf: "center", display: "flex", flexDirection: "column", gap: ".5em" }}>
                <Typography variant={"h5"}>Connect to a session</Typography>
                <TextField
                    value={sessionId}
                    onChange={event => setSessionId(event.target.value)}
                    placeholder={"Session ID"}
                    sx={{
                        width: "20em",
                    }}
                    size={"small"}
                />
                <Button
                    sx={{ alignSelf: "right", marginLeft: "auto", pl: 2, pr: 2 }}
                    onClick={() => navigate(`/session/${encodeURIComponent(sessionId)}`)}
                    disabled={sessionId === ""}
                >
                    Connect
                </Button>
            </Box>
        </Box>
    )
});
