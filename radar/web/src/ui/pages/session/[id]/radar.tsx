import { Box, Typography } from "@mui/material";
import * as React from "react";
import { useContext, useState } from "react";
import ImageBomb from "../../../../assets/bomb.png";
import { kDefaultRadarState } from "../../../../backend/connection";
import { RadarPlayerPawn, RadarState } from "../../../../backend/definitions";
import { LoadedMap, loadMap } from "../../../../map-info";
import { useAppSelector } from "../../../../state";
import BombIndicator from "../../../components/bomb/bomb-indicator";
import IconPlayerDead from "./icon_player_dead.svg";
import IconPlayer from "./icon_player.svg";

export const ContextRadarState = React.createContext<RadarState>(kDefaultRadarState);
const ContextMap = React.createContext<LoadedMap>(null);

export const RadarRenderer = React.memo(() => {
    const { worldName, plantedC4 } = React.useContext(ContextRadarState);
    const [mapInfo, setMapInfo] = React.useState<LoadedMap>(null);
    const showBombDetails = useAppSelector((state) => state.radarSettings.displayBombDetails);

    React.useEffect(() => {
        let obsolete = false;
        loadMap(worldName)
            .then((info) => {
                if (obsolete) {
                    /* no need to update this info anymore */
                    return;
                }

                setMapInfo(info);
            })
            .catch((error) => {
                console.error(`Failed to load ${worldName}`);
                console.error(error);
            });

        return () => {
            obsolete = true;
        };
    }, [worldName]);

    return (
        <ContextMap.Provider value={mapInfo}>
            <Box
                sx={{
                    height: "100%",
                    width: "100%",

                    display: "flex",
                    flexDirection: "column",

                    p: 3,
                }}
            >
                <Typography variant={"h5"}>{mapInfo?.displayName ?? worldName}</Typography>
                <Box
                    sx={{
                        height: "100%",
                        width: "100%",

                        display: "flex",
                        flexDirection: "row",

                        position: "relative",
                        p: 3,
                    }}
                >
                    <Box
                        sx={{
                            position: "absolute",
                            zIndex: 1,

                            top: "1em",
                            left: 0,
                            right: 0,

                            display: "flex",
                            flexDirection: "row",
                            justifyContent: "center",
                        }}
                    >
                        {showBombDetails && plantedC4 && <BombIndicator state={plantedC4.state} />}
                    </Box>
                    <SqareContainer>
                        <MapRenderer />
                        {!mapInfo && (
                            <Box
                                sx={{
                                    position: "absolute",
                                    top: 0,
                                    left: 0,
                                    right: 0,
                                    bottom: 0,
                                    display: "flex",
                                    flexDirection: "column",
                                    justifyContent: "center",
                                }}
                            >
                                <Typography variant={"h5"} sx={{ alignSelf: "center", color: "grey.500" }}>
                                    loading map info
                                </Typography>
                            </Box>
                        )}
                    </SqareContainer>
                </Box>
            </Box>
        </ContextMap.Provider>
    );
});

const SqareContext = React.createContext<number>(1);
const SqareContainer = React.memo((props: { children: React.ReactNode }) => {
    const [sqareSize, setSqareSize] = useState(1);
    const refContainer = React.useRef<HTMLDivElement>();
    const observer = React.useMemo(() => {
        return new ResizeObserver((events) => {
            const event = events[events.length - 1];
            const { width, height } = event.contentRect;
            const sqareSize = Math.min(width, height);
            setSqareSize(sqareSize);
        });
    }, [setSqareSize]);

    React.useEffect(() => {
        if (!refContainer.current) {
            return;
        }

        observer.observe(refContainer.current);
        return () => observer.disconnect();
    }, [refContainer, observer]);

    return (
        <Box sx={{ height: "100%", width: "100%", display: "flex", flexDirection: "column" }} ref={refContainer}>
            <Box
                sx={{
                    marginTop: "auto",
                    marginLeft: "auto",
                    marginRight: "auto",
                    marginBottom: "auto",
                }}
                style={
                    {
                        width: `${sqareSize}px`,
                        height: `${sqareSize}px`,
                    } as any
                }
            >
                <SqareContext.Provider value={sqareSize}>{props.children}</SqareContext.Provider>
            </Box>
        </Box>
    );
});

const MapRenderer = React.memo(() => {
    const { playerPawns, c4Entities, plantedC4 } = React.useContext(ContextRadarState);
    const map = React.useContext(ContextMap);

    const { colorDotCT, colorDotT, colorDotOwn } = useAppSelector((state) => state.radarSettings);
    return (
        <Box
            sx={{
                position: "relative",

                height: "100%",
                width: "100%",

                ".icon_player_svg__view-cone": {
                    fill: "#fff",
                },
                ".team-t": {
                    ".icon_player_svg__player-dot, .icon_player_dead_svg__player_cross": {
                        fill: colorDotT,
                    },
                },
                ".team-ct": {
                    ".icon_player_svg__player-dot, .icon_player_dead_svg__player_cross": {
                        fill: colorDotCT,
                    },
                },
                ".broadcaster": {
                    ".icon_player_svg__player-dot, .icon_player_dead_svg__player_cross": {
                        fill: colorDotOwn,
                    },
                },
            }}
        >
            <Box
                sx={{
                    height: "100%",
                    width: "100%",
                    backgroundImage: `url("${map?.overlayRadar}")`,
                    backgroundPosition: "center",
                    backgroundSize: "contain",
                }}
            />
            {playerPawns.map((pawn) => (
                <MapPlayerPawn playerInfo={pawn} key={`player-${pawn.pawnEntityId}`} />
            ))}
            {c4Entities.map((entity) => (
                <MapC4 position={entity.position} key={`c4-${entity.entityId}`} />
            ))}
            {plantedC4 && <MapC4 position={plantedC4.position} key="planted-c4" />}
        </Box>
    );
});

const useMapPosition = (position: [number, number, number]): [number, number] | null => {
    const map = React.useContext(ContextMap);
    if (!map) {
        /* we need the map info */
        return null;
    }

    const { metaInfo } = map;
    const offsets = metaInfo.offset;
    const mapSize = metaInfo.resolution * 1024;

    const floorOffset = map.metaInfo.floors.find(
        (floor) => floor.zRange.min <= position[2] && position[2] <= floor.zRange.max,
    )?.offset ?? {
        x: 0,
        y: 0,
    };
    return [
        ((position[0] + offsets.x) * 100) / mapSize + floorOffset.x,
        ((position[1] + offsets.y) * 100) / mapSize + floorOffset.y,
    ];
};

export const MapPlayerPawn = React.memo((props: { playerInfo: RadarPlayerPawn }) => {
    const showOwn = useAppSelector((state) => state.radarSettings.showDotOwn);
    const { localControllerEntityId } = useContext(ContextRadarState);
    const { playerInfo } = props;
    const playerPosition = useMapPosition(playerInfo.position) ?? [0, 0];
    return (
        <MapPlayerIcon
            position={playerPosition}
            rotation={playerInfo.playerHealth <= 0 ? 0 : playerInfo.rotation * -1}
            team={playerInfo.teamId === 3 ? "ct" : "t"}
            health={playerInfo.playerHealth}
            isBroadcaster={showOwn && playerInfo.controllerEntityId === localControllerEntityId}
        />
    );
});

export const MapPlayerIcon = (props: {
    position: [number, number];
    rotation: number;

    team: "t" | "ct";
    health: number;

    isBroadcaster: boolean;
}) => {
    const { position, health, rotation, isBroadcaster } = props;
    const mapWidth = useContext(SqareContext);
    const iconSize = useAppSelector((state) => state.radarSettings.iconSize);
    const iconWidth = (mapWidth * iconSize) / 100;

    let Icon;
    if (health <= 0) {
        Icon = IconPlayerDead;
    } else {
        Icon = IconPlayer;
    }

    return (
        <Icon
            style={{
                position: "absolute",

                bottom: `${(position[1] * mapWidth) / 100 - iconWidth / 2}px`,
                left: `${(position[0] * mapWidth) / 100 - iconWidth / 2}px`,

                rotate: `${rotation + 90}deg`,
                filter: "drop-shadow(-2px -2px 3px rgba(0, 0, 0, .5))",
            }}
            width={iconWidth}
            className={`team-${props.team} ${isBroadcaster ? "broadcaster" : ""}`}
        />
    );
};

const MapC4 = React.memo((props: { position: [number, number, number] }) => {
    const { position } = props;
    const [bombX, bombY] = useMapPosition(position) ?? [0, 0];

    const iconSize = useAppSelector((state) => state.radarSettings.iconSize);
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
            style={
                {
                    "--pos-x": `${bombX - iconSize / 2}%`,
                    "--pos-y": `${bombY - iconSize / 2}%`,
                } as any
            }
        />
    );
});
