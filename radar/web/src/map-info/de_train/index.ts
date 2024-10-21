import { LoadedMap } from "..";
import SimpleRadarDefault from "./radar_0_default.png";

export default {
    mapName: "de_train",
    displayName: "Train",

    pos_x: -2510, // upper left world coordinate
    pos_y: 2440,
    scale: 4.74,

    verticalSections: {
        default: // use the primary radar image
        {
            altitudeMax: 10000,
            altitudeMin: -10000,
        },
    },

    mapImages: [
        {
            name: "SimpleRadar",
            images:{
                default: SimpleRadarDefault,
            }
        },
    ]
} satisfies LoadedMap;
