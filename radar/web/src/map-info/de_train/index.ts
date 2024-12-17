import {LoadedMap} from "..";
import OfficialDefault from "./map_style_cs2.png";

export default {
    mapName: "de_train",
    displayName: "Train",

    pos_x: -2308, // upper left world coordinate
    pos_y: 2078,
    scale: 4.082077,

    verticalSections: [
        {
            name: "default",
            altitudeMax: 10000,
            altitudeMin: -10000,
        }
    ],

    mapStyles: [
        {
            name: "Official",
            map: {
                default: OfficialDefault,
            }
        }
    ]
} satisfies LoadedMap;


