import { Box, Button, Dialog, DialogActions, DialogContent, DialogTitle, Modal, Slider, Typography } from "@mui/material";
import { useAppDispatch, useAppSelector } from "../../../../state"
import { updateRadarSettings } from "../../../../state/radar-settings";
import React from "react";
import { MapPlayerIcon, MapPlayerPing } from "./radar";

export default React.memo(() => {
    const isOpen = useAppSelector(state => state.radarSettings.dialogOpen);
    const dispatch = useAppDispatch();

    return (
        <Dialog
            open={isOpen}
            onClose={() => dispatch(updateRadarSettings({ dialogOpen: false }))}
        >
            <DialogTitle>
                Radar Settings
            </DialogTitle>
            <DialogContent sx={{ width: "15em", height: "10em", overflow: "visible" }}>
                <SettingIconSize />
            </DialogContent>
            <DialogActions>
                <Button onClick={() => dispatch(updateRadarSettings({ dialogOpen: false }))}>Close</Button>
            </DialogActions>
        </Dialog>
    );
});

const SettingIconSize = React.memo(() => {
    const value = useAppSelector(state => state.radarSettings.iconSize);
    const dispatch = useAppDispatch();

    return (
        <Box>
            <Typography variant={"subtitle1"}>Icon Size</Typography>
            <Slider
                min={0.1}
                max={5.0}
                step={0.1}

                value={value}
                onChange={(_event, value) => dispatch(updateRadarSettings({ iconSize: value }))}

                valueLabelDisplay={"auto"}
            />
        </Box>
    );
})