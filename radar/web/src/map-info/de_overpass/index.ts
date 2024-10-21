import { LoadedMap } from "..";
import SimpleRadarDefault from "./radar_0_default.png";
import OfficialDefault from "./radar_1_default.png";

export default {
    mapName: "de_overpass",
    displayName: "Overpass",

    pos_x: -4831, // upper left world coordinate
    pos_y: 1781,
    scale: 5.2,

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
        {
            name: "Official",
            images:{
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;
