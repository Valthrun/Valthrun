import { Box, Paper, Typography } from "@mui/material";
import * as colors from "@mui/material/colors";
import React from "react";
import { PlantedC4State } from "../../../backend/definitions";
import IconC4 from "./icon_c4.svg";
import IconDefuse from "./icon_defuse.svg";

const StateBackground = (props: { state: PlantedC4State }) => {
    const { state } = props;

    let background, progress;
    switch (state.state) {
        case "active":
            if (state.defuser !== null) {
                progress = (state.defuser.timeTotal - state.defuser.timeRemaining) / state.defuser.timeTotal;
                background = colors.blue[900];
            } else {
                progress = (state.timeTotal - state.timeDetonation) / state.timeTotal;
                background = colors.red[900];
            }
            break;

        case "defused":
            progress = 1;
            background = colors.blue[900];
            break;

        case "detonated":
            progress = 1;
            background = colors.red[900];
            break;

        default:
            return null;
    }

    return (
        <Box
            sx={{
                position: "absolute",
                zIndex: 1,

                top: 0,
                left: 0,
                bottom: 0,

                background,
            }}
            width={`${(progress * 100).toFixed(0)}%`}
        />
    );
};

const formatTime = (time: number): string => {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time) - minutes * 60;
    const millis = time - Math.floor(time);
    if (minutes > 0) {
        return `${`${minutes}`.padStart(2, "0")}:${`${seconds}`.padStart(2, "0")}:${`${Math.round(millis * 100)}`.padStart(2, "0")}`;
    } else {
        return `${`${seconds}`.padStart(2, "0")}:${`${Math.round(millis * 100)}`.padStart(2, "0")}`;
    }
};

export default React.memo((props: { state: PlantedC4State }) => {
    const { state } = props;

    let text, textColor;
    let Icon;
    switch (state.state) {
        case "active":
            if (state.defuser !== null) {
                text = formatTime(state.defuser.timeRemaining);
                Icon = IconDefuse;

                if (state.defuser.timeRemaining < state.timeDetonation) {
                    textColor = colors.green[700];
                } else {
                    textColor = colors.red[700];
                }
            } else {
                text = formatTime(state.timeDetonation);
                Icon = IconC4;
                textColor = "#FFFFFF";
            }
            break;

        case "defused":
            text = "defused";
            Icon = IconDefuse;
            textColor = colors.green[700];
            break;

        case "detonated":
            text = "detonated";
            Icon = IconC4;
            textColor = "#FFFFFF";
            break;
    }

    return (
        <Paper
            variant="outlined"
            sx={{
                width: "12em",
                height: "3em",

                position: "relative",
                overflow: "hidden",
            }}
        >
            <Box
                sx={{
                    position: "absolute",
                    zIndex: 2,

                    top: 0,
                    left: 0,
                    right: 0,
                    bottom: 0,

                    display: "flex",
                    flexDirection: "row",

                    paddingLeft: 1,
                    paddingRight: 1,

                    "> *": {
                        alignSelf: "center",
                    },
                }}
            >
                <Icon width="2em" height="2em" fill={textColor} />
                <Typography variant="h6" sx={{ marginLeft: "auto", marginRight: "auto", color: textColor }}>
                    {text}
                </Typography>
            </Box>
            <StateBackground state={state} />
        </Paper>
    );
});
