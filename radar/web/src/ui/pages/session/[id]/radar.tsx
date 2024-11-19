import { Box, SxProps, Theme, Typography } from "@mui/material";
import * as React from "react";
import { useContext, useState } from "react";
import { kDefaultRadarState } from "../../../../backend/connection";
import { LoadedMap, loadMap, MapStyle } from "../../../../map-info";
import ImageBomb from "../../../../assets/bomb.png";
import { useAppSelector } from "../../../../state";
import BombIndicator from "../../../components/bomb/bomb-indicator";
import IconPlayerDead from "./icon_player_dead.svg";
import IconPlayer from "./icon_player.svg";
import { F32, RadarPlayerPawn, RadarState } from "../../../../backend/definitions";
import SizedContainer from "../../../components/container/sized-container";
import SqareContainer, { useSqareSize } from "../../../components/container/sqare-container";
import { useQuery } from "react-query";

export const ContextRadarState = React.createContext<RadarState>(kDefaultRadarState);
const ContextMap = React.createContext<LoadedMap>(null);

export const RadarRenderer = React.memo(() => {
    const { worldName, plantedC4 } = React.useContext(ContextRadarState);
    const isInMatch = !worldName.includes("empty");

    const queryMap = useQuery({
        queryKey: ["map-info", worldName],
        queryFn: async () => {
            return await loadMap(worldName);
        },
        enabled: isInMatch
    });

    const displayBombDetails = useAppSelector(state => state.radarSettings.displayBombDetails);

    return (
        <ContextMap.Provider value={queryMap.data ?? null}>
            <Box
                sx={{
                    height: "100%",
                    width: "100%",

                    display: "flex",
                    flexDirection: "column",

                    p: 3,
                }}
            >
                <Typography variant={"h5"}>{queryMap.data?.displayName ?? worldName}</Typography>
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
                        {displayBombDetails && plantedC4 && <BombIndicator state={plantedC4.state} />}
                    </Box>

                    {isInMatch && queryMap.isSuccess && (
                        queryMap.data ? (
                            <MapContainer />
                        ) : (
                            <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                                <Typography variant={"h5"} sx={{ alignSelf: "center", color: "error.dark" }}>
                                    Map Unknown
                                </Typography>
                            </Box>
                        )
                    )}
                    {isInMatch && (queryMap.isLoading || queryMap.isError) && (
                        <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                            {queryMap.isLoading ? (
                                <Typography variant={"h5"} sx={{ alignSelf: "center", color: "grey.500" }}>
                                    loading map info
                                </Typography>
                            ) : (
                                <Typography variant={"h5"} sx={{ alignSelf: "center", color: "palette.error.dark" }}>
                                    <React.Fragment>
                                        Failed to load map.<br />
                                        Lookup the console for more details.
                                    </React.Fragment>
                                </Typography>
                            )}
                        </Box>
                    )}
                    {!isInMatch && (
                        <Box sx={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, display: "flex", flexDirection: "column", justifyContent: "center" }}>
                            <Typography variant={"h5"} sx={{ alignSelf: "center", color: "grey.500" }}>
                                waiting for match
                            </Typography>
                        </Box>
                    )}
                </Box>
            </Box>
        </ContextMap.Provider>
    );
});

const MapContainer = React.memo(() => {
    const map = React.useContext(ContextMap);
    const { localControllerEntityId, playerPawns } = React.useContext(ContextRadarState);
    const showAllLayers = useAppSelector(state => state.radarSettings.showAllLayers);
    if (!map) {
        return null;
    }

    const localPlayerPosition = playerPawns.find(pawn => pawn.controllerEntityId === localControllerEntityId)?.position ?? [0, 0, 0];
    const localMapLevel = getMapLevel(map, localPlayerPosition);

    return (
        <SizedContainer sx={{
            display: "flex",
            flexDirection: "column",
            justifyContent: "center"
        }}>
            {size => {
                if (showAllLayers && map.verticalSections.length > 1) {
                    const minAxis = Math.min(size.width, size.height);
                    const maxAxis = Math.max(size.width, size.height);

                    const sqareSize = Math.min(minAxis, maxAxis / 2);
                    return (
                        <Box sx={{
                            display: "flex",
                            flexDirection: "row",
                            flexWrap: "wrap",

                            justifyContent: "center"
                        }}>
                            {map.verticalSections.map(section => (
                                <SqareContainer sqareSize={sqareSize} key={section.name}>
                                    <MapLevel level={section.name} />
                                </SqareContainer>
                            ))}
                        </Box>
                    )
                } else {
                    const minAxis = Math.min(size.width, size.height);
                    return (
                        <SqareContainer sqareSize={minAxis} >
                            <MapLevel level={localMapLevel} />
                        </SqareContainer>
                    );
                }

            }}
        </SizedContainer>
    );
});

const MapLevel = React.memo((props: { level: string }) => {
    const { level } = props;

    const map = React.useContext(ContextMap);
    const { playerPawns, c4Entities, plantedC4 } = React.useContext(ContextRadarState);
    const { colorDotCT, colorDotT, colorDotOwn, mapStyle } = useAppSelector(state => state.radarSettings);

    if (!map) {
        /* we need the map info */
        return null;
    }

    const mapImage = map.mapStyles.find(style => style.name === mapStyle) ?? map.mapStyles[0] ?? null;
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
                    backgroundImage: `url("${mapImage?.map[level as keyof typeof mapImage.map]}")`,
                    backgroundPosition: "center",
                    backgroundSize: "cover",
                }}
            />
            {playerPawns.filter(pawn => getMapLevel(map, pawn.position) === level).map(pawn => <MapPlayerPawn playerInfo={pawn} key={`player-${pawn.pawnEntityId}`} />)}
            {c4Entities.filter(entity => getMapLevel(map, entity.position) === level).map(entity => <MapC4 position={entity.position} key={`c4-${entity.entityId}`} />)}
            {plantedC4 && getMapLevel(map, plantedC4.position) === level ? <MapC4 position={plantedC4.position} key="planted-c4" /> : null}
        </Box>
    );
});

const getMapLevel = (map: LoadedMap, position: [F32, F32, F32]): string => {
    return map.verticalSections.find(section => section.altitudeMin <= position[2] && position[2] < section.altitudeMax)?.name ?? "default";
}

const useMapPosition = (position: [number, number, number]): [number, number] | null => {
    const map = React.useContext(ContextMap);
    if (!map) {
        /* we need the map info */
        return null;
    }

    const mapSize = map.scale * 1024;
    return [
        (position[0] - map.pos_x) * 100 / mapSize,
        (position[1] - map.pos_y) * 100 / -mapSize
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
    const mapWidth = useSqareSize();
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

                top: `${position[1] * mapWidth / 100 - iconWidth / 2}px`,
                left: `${position[0] * mapWidth / 100 - iconWidth / 2}px`,

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
                top: "var(--pos-y)",
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
