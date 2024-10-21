import { LoadedMap } from "..";
import OfficialDefault from "./radar_1_default.png";

export default {
    mapName: "de_thera",
    displayName: "Thera",

    pos_x: -85, // upper left world coordinate
    pos_y: 2261,
    scale: 4.8,

    verticalSections: {
        default: // use the primary radar image
        {
            altitudeMax: 10000,
            altitudeMin: -10000,
        },
    },

    mapImages: [
        {
            name: "Official",
            images:{
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;
