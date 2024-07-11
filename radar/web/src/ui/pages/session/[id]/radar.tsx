import * as React from "react";
import { RadarPlayerInfo, RadarBombInfo, RadarState } from "../../../../backend/connection";
import { LoadedMap, loadMap } from "../../../../map-info";
import { Box, Typography } from "@mui/material";
import ImageBlueCross from "../../../../assets/blue_cross.png";
import ImageBlueDot from "../../../../assets/blue_dot.png";
import ImageYellowCross from "../../../../assets/yellow_cross.png";
import ImageYellowDot from "../../../../assets/yellow_dot.png";
import ImageBomb from "../../../../assets/bomb.png";
import { useAppSelector } from "../../../../state";

export const ContextRadarState = React.createContext<RadarState>({
    players: [],
    worldName: "de_anubis",
    bomb: null,
});


const ContextMap = React.createContext<LoadedMap>(null);
export const RadarRenderer = React.memo(() => {
    const { worldName } = React.useContext(ContextRadarState);
    const [mapInfo, setMapInfo] = React.useState<LoadedMap>(null);

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
                <SqareContainer>
                    <MapRenderer />
                    {!mapInfo && (
                        <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                            <Typography variant={"h5"} sx={{ alignSelf: "center", color: "grey.500" }}>loading map info</Typography>
                        </Box>
                    )}
                </SqareContainer>
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
    const { players, bomb } = React.useContext(ContextRadarState);
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
            {players.map(player => <MapPlayerPing playerInfo={player} key={`player-${player.controllerEntityId}`} />)}
            <MapBombPing bombInfo={bomb} />
        </Box>
    )
});

export const MapPlayerPing = React.memo((props: {
    playerInfo: RadarPlayerInfo
}) => {
    const { playerInfo } = props;
    const map = React.useContext(ContextMap);
    if (!map) {
        /* we need the map info */
        return null;
    }

    const offsets = map.metaInfo.offset;
    const mapSize = map.metaInfo.resolution * 1024;

    const [floor] = map.metaInfo.floors.filter(floor => floor.zRange.min <= props.playerInfo.position[2] && props.playerInfo.position[2] <= floor.zRange.max);

    const playerX = props.playerInfo.position[0] + offsets.x;
    const playerY = props.playerInfo.position[1] + offsets.y;

    return (
        <MapPlayerIcon
            posX={playerX * 100 / mapSize + (floor?.offset.x ?? 0)}
            posY={playerY * 100 / mapSize + (floor?.offset.y ?? 0)}
            rotation={playerInfo.playerHealth <= 0 ? 0 : playerInfo.rotation * -1}

            teamId={playerInfo.teamId}
            playerHealth={playerInfo.playerHealth}
        />
    )
});

export const MapPlayerIcon = (props: {
    posX: number,
    posY: number,
    rotation: number,

    teamId: number,
    playerHealth: number,

    size?: number
}) => {
    const iconSize = useAppSelector(state => state.radarSettings.iconSize) * (props.size ?? 1.0);

    let iconSrc;
    if (props.playerHealth <= 0) {
        if (props.teamId === 3) {
            iconSrc = ImageBlueCross;
        } else {
            iconSrc = ImageYellowCross;
        }
    } else {
        if (props.teamId === 3) {
            iconSrc = ImageBlueDot;
        } else {
            iconSrc = ImageYellowDot;
        }
    }

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
                "--pos-x": `${props.posX - iconSize / 2}%`,
                "--pos-y": `${props.posY - iconSize / 2}%`,
                "--rotation": `${props.rotation}deg`
            } as any}
        />
    );
};

const MapBombPing = React.memo((props: {
    bombInfo: RadarBombInfo,
}) => {
    const map = React.useContext(ContextMap);
    const iconSize = useAppSelector(state => state.radarSettings.iconSize);
    if (!map || !props.bombInfo) {
        /* we need the map and bomb info */
        return null;
    }

    let bombX = props.bombInfo.position[0];
    let bombY = props.bombInfo.position[1];

    const mapSize = map.metaInfo.resolution * 1024;
    const offsets = map.metaInfo.offset;
    bombX += offsets.x;
    bombY += offsets.y;

    const [floor] = map.metaInfo.floors.filter(floor => floor.zRange.min <= props.bombInfo.position[2] && props.bombInfo.position[2] <= floor.zRange.max);

    return (
        <Box
            sx={{
                bottom: "var(--pos-y)",
                left: "var(--pos-x)",

                height: `${iconSize}%`,
                width: `${iconSize}%`,

                position: "absolute",

                backgroundImage: `url("${ImageBomb}")`,
                backgroundPosition: "center",
                backgroundSize: "contain",
            }}

            style={{
                "--pos-x": `${bombX * 100 / mapSize - iconSize / 2 + (floor?.offset.x ?? 0)}%`,
                "--pos-y": `${bombY * 100 / mapSize - iconSize / 2 + (floor?.offset.y ?? 0)}%`,
            } as any}
        />
    )
});