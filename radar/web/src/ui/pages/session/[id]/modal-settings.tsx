import { SettingsBackupRestore as IconReset } from "@mui/icons-material";
import {
    Box,
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    IconButton,
    Slider,
    Switch,
    Typography,
    InputLabel,
    MenuItem,
    FormControl,
    Select
} from "@mui/material";
import { MuiColorInput } from "mui-color-input";
import React from "react";
import { useAppDispatch, useAppSelector } from "../../../../state";
import { kDefaultRadarSettings, RadarSettingsState, updateRadarSettings } from "../../../../state/radar-settings";

export default React.memo(() => {
    const isOpen = useAppSelector((state) => state.radarSettings.dialogOpen);
    const dispatch = useAppDispatch();
    const highlightBroadcaster = useAppSelector((state) => state.radarSettings.showDotOwn);

    return (
        <Dialog open={isOpen} onClose={() => dispatch(updateRadarSettings({ dialogOpen: false }))}>
            <DialogTitle>Radar Settings</DialogTitle>
            <DialogContent
                sx={{
                    minWidth: "15em",
                    width: "25em",

                    minHeight: "10em",
                    overflow: "auto",
                }}
            >
                <SettingStyleSelector />

                <SettingIconSize />

                <SettingBoolean target="displayBombDetails" title="Display Bomb Details" />
                <SettingBoolean target="showAllLayers" title="Display all levels" />
                <SettingBoolean target="showDotOwn" title="Highlight broadacster" />

                <SettingDotColor target="colorDotCT" title="CT Color" />
                <SettingDotColor target="colorDotT" title="T Color" />
                {highlightBroadcaster && <SettingDotColor target="colorDotOwn" title="Own Color" />}
            </DialogContent>
            <DialogActions>
                <Button onClick={() => dispatch(updateRadarSettings({ dialogOpen: false }))}>Close</Button>
            </DialogActions>
        </Dialog>
    );
});

const SettingIconSize = React.memo(() => {
    const value = useAppSelector((state) => state.radarSettings.iconSize);
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
});

const SettingBoolean = React.memo((props: {
    title: string,
    target: keyof RadarSettingsState & ("displayBombDetails" | "showDotOwn" | "showAllLayers")
}) => {
    const { target, title } = props;
    const value = useAppSelector(state => state.radarSettings[target]);
    const dispatch = useAppDispatch();

    return (
        <Box>
            <Typography variant={"subtitle1"}>{title}</Typography>
            <Switch
                checked={value}
                onChange={(_event, value) => dispatch(updateRadarSettings({ [target]: value }))}
            />
        </Box>
    );
},
);

const SettingDotColor = React.memo(
    (props: { title: string; target: keyof RadarSettingsState & ("colorDotCT" | "colorDotT" | "colorDotOwn") }) => {
        const value = useAppSelector((state) => state.radarSettings[props.target]);
        const dispatch = useAppDispatch();

        return (
            <Box>
                <Typography variant={"subtitle1"}>{props.title}</Typography>
                <Box
                    sx={{
                        display: "flex",
                        flexDirection: "row",
                        gap: "1em",
                    }}
                >
                    <MuiColorInput
                        fullWidth
                        sx={{ minWidth: "5em" }}
                        size="small"
                        format="hex"
                        value={value}
                        onChange={(event) =>
                            dispatch(
                                updateRadarSettings({
                                    [props.target]: event,
                                }),
                            )
                        }
                    />
                    <IconButton
                        onClick={() => {
                            dispatch(
                                updateRadarSettings({
                                    [props.target]: kDefaultRadarSettings[props.target],
                                }),
                            );
                        }}
                        title="Reset value"
                    >
                        <IconReset />
                    </IconButton>
                </Box>
            </Box>
        );
    },
);

const SettingStyleSelector = React.memo(() => {
    const value = useAppSelector((state) => state.radarSettings.mapStyle);
    const dispatch = useAppDispatch();

    return (
        <Box>
            <Typography variant={"subtitle1"}>Radar Style</Typography>
            <FormControl fullWidth>
                <Select
                    value={value}
                    onChange={event => dispatch(updateRadarSettings({ mapStyle: event.target.value }))}
                >
                    <MenuItem value="Official">Official</MenuItem>
                    <MenuItem value="SimpleRadar">Simple Radar</MenuItem>
                </Select>
            </FormControl>
        </Box>
    );
});
