import { LoadedMap } from "..";
import SimpleRadarDefault from "./map_style_simple_radar.png";

export default {
    mapName: "de_train",
    displayName: "Train",

    pos_x: -2510, // upper left world coordinate
    pos_y: 2440,
    scale: 4.74,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "SimpleRadar",
            map: {
                default: SimpleRadarDefault,
            }
        },
    ]
} satisfies LoadedMap;
