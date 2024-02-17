import * as React from "react";
import { RadarPlayerInfo, RadarState } from "../../../../backend/connection";
import { LoadedMap, loadMap } from "../../../../map-info";
import {Box, Drawer, IconButton, Switch, FormControlLabel, Typography, Slider} from "@mui/material";
import ImageBlueCross from "../../../../assets/blue_cross.png";
import ImageBlueDot from "../../../../assets/blue_dot.png";
import ImageYellowCross from "../../../../assets/yellow_cross.png";
import ImageYellowDot from "../../../../assets/yellow_dot.png";
import MenuIcon from "@mui/icons-material/Menu";

export const ContextRadarState = React.createContext<RadarState>({
    players: [],
    worldName: "de_anubis"
});


const ContextMap = React.createContext<LoadedMap>(null);
export const RadarRenderer = React.memo(() => {
    const { worldName } = React.useContext(ContextRadarState);
    const [mapInfo, setMapInfo] = React.useState<LoadedMap>(null);
    const [drawerOpen, setDrawerOpen] = React.useState(false);
    const [iconSize, setIconSize] = React.useState(3.125);
    const [featureXEnabled, setFeatureXEnabled] = React.useState(false);

    const toggleDrawer = () => {
        setDrawerOpen(!drawerOpen);
    };

    React.useEffect(() => {
        let obsolete = false;
        loadMap(worldName)
            .then(info => {
                if (obsolete) {
                    /* no need to update this info anymore */
                    return;
                }

                setMapInfo(info);
            })
            .catch(error => {
                console.error(`Failed to load ${worldName}`);
                console.error(error);
            })

        return () => {
            obsolete = true;
        }
    }, [worldName]);

    return (
        <ContextMap.Provider value={mapInfo}>
            <Box sx={{
                height: "100%",
                width: "100%",

                display: "flex",
                flexDirection: "column",

                p: 3,
            }}>
                <Typography variant={"h5"}>{mapInfo?.displayName ?? worldName}</Typography>
                <IconButton onClick={toggleDrawer} sx={{ position: 'absolute', top: 0, left: 0 }}>
                    <MenuIcon />
                </IconButton>
                <Drawer
                    anchor={'right'}
                    open={drawerOpen}
                    onClose={toggleDrawer}
                >
                    <Box sx={{ width: 250 }}>
                        <Box sx={{ paddingX: 2 }}>
                            <FormControlLabel
                                control={
                                    <Switch
                                        checked={featureXEnabled}
                                        onChange={(event) => setFeatureXEnabled(event.target.checked)}
                                    />
                                }
                                label="Feature X"
                            />
                            <Typography>Icon Size</Typography>
                            <Slider
                                value={iconSize}
                                onChange={(event, newValue) => {
                                    if (typeof newValue === 'number') {
                                        setIconSize(newValue)
                                    }
                                }}
                                step={0.1}
                                min={1}
                                max={5}
                                valueLabelDisplay="auto"
                            />
                        </Box>
                    </Box>
                </Drawer>
                <IconSizeContext.Provider value={{ iconSize }}>
                    <SqareContainer>
                        <MapRenderer />
                        {!mapInfo && (
                            <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                                <Typography variant={"h5"} sx={{ alignSelf: "center", color: "grey.500" }}>loading map info</Typography>
                            </Box>
                        )}
                    </SqareContainer>
                </IconSizeContext.Provider>
            </Box>
        </ContextMap.Provider>
    );
});

const SqareContainer = React.memo((props: {
    children: React.ReactNode,
}) => {
    const refInner = React.useRef<HTMLDivElement>();
    const refContainer = React.useRef<HTMLDivElement>();
    const observer = React.useMemo(() => {
        return new ResizeObserver(events => {
            const inner = refInner.current;
            if (!inner) {
                return;
            }

            const event = events[events.length - 1];
            const { width, height } = event.contentRect;
            const sqareSize = Math.min(width, height);

            inner.style.left = `${(width - sqareSize) / 2}px`;
            inner.style.top = `${(height - sqareSize) / 2}px`;

            inner.style.width = `${sqareSize}px`;
            inner.style.height = `${sqareSize}px`;
        });
    }, []);

    React.useEffect(() => {
        if (!refContainer.current) {
            return;
        }

        observer.observe(refContainer.current);
        return () => observer.disconnect();
    }, [refContainer]);

    return (
        <Box sx={{ position: "relative", height: "100%", width: "100%" }} ref={refContainer}>
            <Box
                sx={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    height: "100%",
                }}
                ref={refInner}
            >
                {props.children}
            </Box>
        </Box>
    )
});

const MapRenderer = React.memo(() => {
    const { players } = React.useContext(ContextRadarState);
    const map = React.useContext(ContextMap);

    return (
        <Box sx={{ position: "relative", height: "100%", width: "100%" }}>
            <Box
                sx={{
                    height: "100%",
                    width: "100%",
                    backgroundImage: `url("${map?.overlayRadar}")`,
                    backgroundPosition: "center",
                    backgroundSize: "contain",
                }}
            />
            {players.map(player => <MapPlayerPing info={player} key={`player-${player.controllerEntityId}`} />)}
        </Box>
    )
});

export const IconSizeContext = React.createContext({
    iconSize: 3.125,
});
const MapPlayerPing = React.memo((props: {
    info: RadarPlayerInfo,
}) => {
    const { info } = props;
    const map = React.useContext(ContextMap);
    if (!map) {
        /* we need the map info */
        return null;
    }

    const { iconSize } = React.useContext(IconSizeContext);
    let iconSrc;
    if (info.playerHealth <= 0) {
        if (info.teamId === 3) {
            iconSrc = ImageBlueCross;
        } else {
            iconSrc = ImageYellowCross;
        }
    } else {
        if (info.teamId === 3) {
            iconSrc = ImageBlueDot;
        } else {
            iconSrc = ImageYellowDot;
        }
    }

    const offsets = map.metaInfo.offset;
    const mapSize = map.metaInfo.resolution * 1024;

    const [floor] = map.metaInfo.floors.filter(floor => floor.zRange.min <= props.info.position[2] && props.info.position[2] <= floor.zRange.max);

    const playerX = props.info.position[0] + offsets.x;
    const playerY = props.info.position[1] + offsets.y;

    return (
        <Box
            sx={{
                bottom: "var(--pos-y)",
                left: "var(--pos-x)",

                height: `${iconSize}%`,
                width: `${iconSize}%`,

                position: "absolute",

                backgroundImage: `url("${iconSrc}")`,
                backgroundPosition: "center",
                backgroundSize: "contain",

                rotate: `var(--rotation)`,
            }}

            style={{
                "--pos-x": `${playerX * 100 / mapSize - iconSize / 2 + (floor?.offset.x ?? 0)}%`,
                "--pos-y": `${playerY * 100 / mapSize - iconSize / 2 + (floor?.offset.y ?? 0)}%`,
                "--rotation": `${info.playerHealth <= 0 ? 0 : info.rotation * -1}deg`
            } as any}
        />
    )
});