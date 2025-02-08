use std::path::Path;
use crate::{components, modify::*, t, utils::{notify, write_log}, window, LinkList, LinkProp, Status, Tab};
use dioxus::prelude::*;

static LOGO_BASE64: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAMAAACdt4HsAAAB5lBMVEUAAADly/+Vf/7MmP7Mmf6Wgf7MmP5/Zf5/ZPm2kf/cp/+VgP6VgP7MmP7MmP7mzP7MmP7MmP7MmP7ly/7Mmf7MmP7Mmf/MmP7Mmf/Nmv+4jv+9jf/mzP7mzP7Mmf6VgP7MmP7mzP6VgP7mzP7MmP+Vf/7MmP6UgP6VgP7Mmf6Vf/+VgP7MmP6Vf/7Mmf6Vf/7mzP7LmP7LmP+Uf//lyv7NmP6WgP+VgP6Wfv7jyvzLmP6Vf/7JmvvqzP7OnP6Ugf/gwv+Pb/6Vf/7Lmf7ly/7Lmf7Mmf+Kcv6VgP6VgP7LmP7my/6UgP7LmP7ly/3lzP6VgP6Vf/7lzP7LmP7LmP3KmP3ly/6UgP6UfvzLmP6Uf/6Vf/yWf//Gr/6vjP6vhf7ky/7ly/6Vgf7Ll/7ny//Mmf/JmP6Vffrq0P7Qmv7kyf7Mmf/mzP+VgP+AZv+Lc/+xjf/iw/+vjP+ehP/ClP/Zs//Qof+Xgf/KmP/Glv/Dlf+bhP+Eaf/hxf/Ysf/QoP+8kf+Uf/+SfP+ReP+Ha//bwv/Suf/Lsv+9pv+1nv+tlv/Ak/+mkP+eiP+yh/+lgP+def+Odv+Wdf+Ib/+Nbv/Wvf/Cqv+3lf+3j/+5jP+4jP+gi/+riv+oif+hhv+qg/+fgf+Scv++edtDAAAAa3RSTlMA/uz92S/57DAMB/ny7+fmxLChk5J0Xk9BMxYS+O7r5uHZ0c/Nxri4sKqekIuIgHVwbmdcWVdRQzsyLikmHhwbDAj99PPz8Oze29DCv76yq6mkopqWhIF/fXxuamZkZGNfT0tKQDw5NSYhE2t4Nt0AAALiSURBVFjDnZf3WxMxGIBDC2jZm5aNIFOWgIBs995773WGIkU7LFosDkRluvd/argn8bteLgPen/u+z31J2kvRukm7U5iblXllfXJJ8ZacVIMwnrF2efvWvGSDMo4r1+R6vOfrMokHPm7Qloe6C2rdVAQft2nJvo7GKvDAJ/Sq3NKbpw/vZArvp4/K5IGik8n0qQU+3izc5NuFuRWcxPm4RbrJah9vE2yyiglMKUHAiLe1bo+hQ/gP8/czebArvybp0eMnWn4sPs0CTYjQ1340+yFFpzHhj2PGJeLnU1ezEf7l9+P/9JNAGZH0G7GgfwZ8FyKAp26EfmD/2AsI1NOAbiMSIP4iBs7RgLjx7LnF/xIk/jK20AMBdSP0ExM/Oo2BdA8ElI3JwKpPF4CyCbGAuvE5aPqL2EqzdmDqLTb9ZZxAp27g1XvTpwsADOsFkt4ETZ8uAFCJtAIv32Hqx3Eix7QCr58yf2nOFrigEUiaxcyfCRtmC7hLA9LHpz7hG/sdYrCXmvTxwZ83yGHCVg6pAlOzGPy/IRJInKFFEfj43eJHJ4lum+GGPPApyPxVPhDbNsOOUhoQn13wFwxKwPpNogjPLvgrIeLaZ2hiAdHZBT86R1Ruhk5hIPuyC3y2APwM/aLAkQcoxWX1FxzfaS7kHChrRwSzQP2VMPH4GRqcA9V9CNEC9cdipsjN0OYYODGCKCl7qf9b8F7udQjsuoaAjRtMf8m0+BnSR/lAzT3QWYF8h+0E7Fcbtvn5aQhxha+iu0WzPVB+3eqywrzBE4GrjSVwcBA5cJUdYX6GYfhUOXn8M85XxCrh/WgfArqyq28hR3KZws9wHGnQAQ43w0UN35cquSOmqH2P+LoYwRlITZ4hJlCv9ouk99yzSn9gtywQ6VH5abWGDLdHFSgwpBxQ+cVueeCUwh+qMOQUKQI5hoL7cr9V5WfJfa9bFWiUB9T/eArlgSxlwCcPdGfK9dQClMA/wt16Ppp+DzoAAAAASUVORK5CYII=";
static RESTORE: &str = "M938.752 512a384 384 0 0 1-384 384 379.306667 379.306667 0 0 1-220.16-69.546667 21.76 21.76 0 0 1-8.96-15.786666 21.333333 21.333333 0 0 1 5.973333-16.64l30.72-31.146667a21.333333 21.333333 0 0 1 26.88-2.56 294.826667 294.826667 0 0 0 165.546667 50.346667 298.666667 298.666667 0 1 0-298.666667-298.666667h100.693334a20.906667 20.906667 0 0 1 15.36 6.4l8.533333 8.533333a21.333333 21.333333 0 0 1 0 30.293334L229.973333 708.266667a21.76 21.76 0 0 1-30.293333 0l-150.613333-151.04a21.333333 21.333333 0 0 1 0-30.293334l8.533333-8.533333a20.906667 20.906667 0 0 1 15.36-6.4h97.792a384 384 0 0 1 768 0z";
static CHANGE: &str = "M1.946 9.315c-.522-.174-.527-.455.01-.634l19.087-6.362c.529-.176.832.12.684.638l-5.454 19.086c-.15.529-.455.547-.679.045L12 14l6-8-8 6-8.054-2.685z";
static MINIMIZED: &str = "M923 571H130.7c-27.6 0-50-22.4-50-50s22.4-50 50-50H923c27.6 0 50 22.4 50 50s-22.4 50-50 50z";
static MAXIMIZE1: &str = "M812.3 959.4H213.7c-81.6 0-148-66.4-148-148V212.9c0-81.6 66.4-148 148-148h598.5c81.6 0 148 66.4 148 148v598.5C960.3 893 893.9 959.4 812.3 959.4zM213.7 120.9c-50.7 0-92 41.3-92 92v598.5c0 50.7 41.3 92 92 92h598.5c50.7 0 92-41.3 92-92V212.9c0-50.7-41.3-92-92-92H213.7z";
static MAXIMIZE2: &str = "M812.2 65H351.6c-78.3 0-142.5 61.1-147.7 138.1-77 5.1-138.1 69.4-138.1 147.7v460.6c0 81.6 66.4 148 148 148h460.6c78.3 0 142.5-61.1 147.7-138.1 77-5.1 138.1-69.4 138.1-147.7V213c0-81.6-66.4-148-148-148z m-45.8 746.3c0 50.7-41.3 92-92 92H213.8c-50.7 0-92-41.3-92-92V350.7c0-50.7 41.3-92 92-92h460.6c50.7 0 92 41.3 92 92v460.6z m137.8-137.7c0 47.3-35.8 86.3-81.8 91.4V350.7c0-81.6-66.4-148-148-148H260.2c5.1-45.9 44.2-81.8 91.4-81.8h460.6c50.7 0 92 41.3 92 92v460.7z";
static CLOSE1: &str = "M109.9 935.8c-19.5-19.5-19.5-51.2 0-70.7l759.3-759.3c19.5-19.5 51.2-19.5 70.7 0s19.5 51.2 0 70.7L180.6 935.8c-19.6 19.6-51.2 19.6-70.7 0z";
static CLOSE2: &str = "M869.1 935.8L109.9 176.5c-19.5-19.5-19.5-51.2 0-70.7s51.2-19.5 70.7 0l759.3 759.3c19.5 19.5 19.5 51.2 0 70.7-19.6 19.6-51.2 19.6-70.8 0z";
static HOME: &str = "M923.733333 394.666667c-85.333333-70.4-206.933333-174.933333-362.666666-309.333334C533.333333 61.866667 490.666667 61.866667 462.933333 85.333333c-155.733333 134.4-277.333333 238.933333-362.666666 309.333334-14.933333 14.933333-25.6 34.133333-25.6 53.333333 0 38.4 32 70.4 70.4 70.4H192v358.4c0 29.866667 23.466667 53.333333 53.333333 53.333333H405.333333c29.866667 0 53.333333-23.466667 53.333334-53.333333v-206.933333h106.666666v206.933333c0 29.866667 23.466667 53.333333 53.333334 53.333333h160c29.866667 0 53.333333-23.466667 53.333333-53.333333V518.4h46.933333c38.4 0 70.4-32 70.4-70.4 0-21.333333-10.666667-40.533333-25.6-53.333333z m-44.8 59.733333h-57.6c-29.866667 0-53.333333 23.466667-53.333333 53.333333v358.4h-138.666667V661.333333c0-29.866667-23.466667-53.333333-53.333333-53.333333h-128c-29.866667 0-53.333333 23.466667-53.333333 53.333333v206.933334H256V507.733333c0-29.866667-23.466667-53.333333-53.333333-53.333333H145.066667c-4.266667 0-6.4-2.133333-6.4-6.4 0-2.133333 2.133333-4.266667 2.133333-6.4 85.333333-70.4 206.933333-174.933333 362.666667-309.333333 4.266667-4.266667 10.666667-4.266667 14.933333 0 155.733333 134.4 277.333333 238.933333 362.666667 309.333333 2.133333 2.133333 2.133333 2.133333 2.133333 4.266667 2.133333 6.4-2.133333 8.533333-4.266667 8.533333z";
static TOOL: &str = "M885.333333 256H725.333333V198.4C723.2 157.866667 689.066667 128 648.533333 128h-298.666666c-40.533333 2.133333-72.533333 34.133333-72.533334 74.666667V256H138.666667C98.133333 256 64 290.133333 64 330.666667v490.666666C64 861.866667 98.133333 896 138.666667 896h746.666666c40.533333 0 74.666667-34.133333 74.666667-74.666667v-490.666666c0-40.533333-34.133333-74.666667-74.666667-74.666667zM341.333333 202.666667c2.133333-6.4 6.4-10.666667 12.8-10.666667h296.533334c6.4 0 10.666667 6.4 10.666666 10.666667V256H341.333333V202.666667zM138.666667 320h746.666666c6.4 0 10.666667 4.266667 10.666667 10.666667v128H128v-128c0-6.4 4.266667-10.666667 10.666667-10.666667z m277.333333 202.666667h192V576c0 6.4-4.266667 10.666667-10.666667 10.666667h-170.666666c-6.4 0-10.666667-4.266667-10.666667-10.666667v-53.333333z m469.333333 309.333333h-746.666666c-6.4 0-10.666667-4.266667-10.666667-10.666667v-298.666666h224V576c0 40.533333 34.133333 74.666667 74.666667 74.666667h170.666666c40.533333 0 74.666667-34.133333 74.666667-74.666667v-53.333333H896v298.666666c0 6.4-4.266667 10.666667-10.666667 10.666667z";
static SETTING1: &str = "M904.533333 422.4l-85.333333-14.933333-17.066667-38.4 49.066667-70.4c14.933333-21.333333 12.8-49.066667-6.4-68.266667l-53.333333-53.333333c-19.2-19.2-46.933333-21.333333-68.266667-6.4l-70.4 49.066666-38.4-17.066666-14.933333-85.333334c-2.133333-23.466667-23.466667-42.666667-49.066667-42.666666h-74.666667c-25.6 0-46.933333 19.2-53.333333 44.8l-14.933333 85.333333-38.4 17.066667L296.533333 170.666667c-21.333333-14.933333-49.066667-12.8-68.266666 6.4l-53.333334 53.333333c-19.2 19.2-21.333333 46.933333-6.4 68.266667l49.066667 70.4-17.066667 38.4-85.333333 14.933333c-21.333333 4.266667-40.533333 25.6-40.533333 51.2v74.666667c0 25.6 19.2 46.933333 44.8 53.333333l85.333333 14.933333 17.066667 38.4L170.666667 727.466667c-14.933333 21.333333-12.8 49.066667 6.4 68.266666l53.333333 53.333334c19.2 19.2 46.933333 21.333333 68.266667 6.4l70.4-49.066667 38.4 17.066667 14.933333 85.333333c4.266667 25.6 25.6 44.8 53.333333 44.8h74.666667c25.6 0 46.933333-19.2 53.333333-44.8l14.933334-85.333333 38.4-17.066667 70.4 49.066667c21.333333 14.933333 49.066667 12.8 68.266666-6.4l53.333334-53.333334c19.2-19.2 21.333333-46.933333 6.4-68.266666l-49.066667-70.4 17.066667-38.4 85.333333-14.933334c25.6-4.266667 44.8-25.6 44.8-53.333333v-74.666667c-4.266667-27.733333-23.466667-49.066667-49.066667-53.333333z m-19.2 117.333333l-93.866666 17.066667c-10.666667 2.133333-19.2 8.533333-23.466667 19.2l-29.866667 70.4c-4.266667 10.666667-2.133333 21.333333 4.266667 29.866667l53.333333 76.8-40.533333 40.533333-76.8-53.333333c-8.533333-6.4-21.333333-8.533333-29.866667-4.266667L576 768c-10.666667 4.266667-17.066667 12.8-19.2 23.466667l-17.066667 93.866666h-57.6l-17.066666-93.866666c-2.133333-10.666667-8.533333-19.2-19.2-23.466667l-70.4-29.866667c-10.666667-4.266667-21.333333-2.133333-29.866667 4.266667l-76.8 53.333333-40.533333-40.533333 53.333333-76.8c6.4-8.533333 8.533333-21.333333 4.266667-29.866667L256 576c-4.266667-10.666667-12.8-17.066667-23.466667-19.2l-93.866666-17.066667v-57.6l93.866666-17.066666c10.666667-2.133333 19.2-8.533333 23.466667-19.2l29.866667-70.4c4.266667-10.666667 2.133333-21.333333-4.266667-29.866667l-53.333333-76.8 40.533333-40.533333 76.8 53.333333c8.533333 6.4 21.333333 8.533333 29.866667 4.266667L448 256c10.666667-4.266667 17.066667-12.8 19.2-23.466667l17.066667-93.866666h57.6l17.066666 93.866666c2.133333 10.666667 8.533333 19.2 19.2 23.466667l70.4 29.866667c10.666667 4.266667 21.333333 2.133333 29.866667-4.266667l76.8-53.333333 40.533333 40.533333-53.333333 76.8c-6.4 8.533333-8.533333 21.333333-4.266667 29.866667L768 448c4.266667 10.666667 12.8 17.066667 23.466667 19.2l93.866666 17.066667v55.466666z";
static SETTING2: &str = "M512 394.666667c-64 0-117.333333 53.333333-117.333333 117.333333s53.333333 117.333333 117.333333 117.333333 117.333333-53.333333 117.333333-117.333333-53.333333-117.333333-117.333333-117.333333z m0 170.666666c-29.866667 0-53.333333-23.466667-53.333333-53.333333s23.466667-53.333333 53.333333-53.333333 53.333333 23.466667 53.333333 53.333333-23.466667 53.333333-53.333333 53.333333z";
static HISTORY1: &str = "M512 74.666667C270.933333 74.666667 74.666667 270.933333 74.666667 512S270.933333 949.333333 512 949.333333 949.333333 753.066667 949.333333 512 753.066667 74.666667 512 74.666667z m0 810.666666c-204.8 0-373.333333-168.533333-373.333333-373.333333S307.2 138.666667 512 138.666667 885.333333 307.2 885.333333 512 716.8 885.333333 512 885.333333z";
static HISTORY2: &str = "M695.466667 567.466667l-151.466667-70.4V277.333333c0-17.066667-14.933333-32-32-32s-32 14.933333-32 32v238.933334c0 12.8 6.4 23.466667 19.2 29.866666l170.666667 81.066667c4.266667 2.133333 8.533333 2.133333 12.8 2.133333 12.8 0 23.466667-6.4 29.866666-19.2 6.4-14.933333 0-34.133333-17.066666-42.666666z";
static ABOUT1: &str = "M512 74.666667C270.933333 74.666667 74.666667 270.933333 74.666667 512S270.933333 949.333333 512 949.333333 949.333333 753.066667 949.333333 512 753.066667 74.666667 512 74.666667z m0 810.666666c-204.8 0-373.333333-168.533333-373.333333-373.333333S307.2 138.666667 512 138.666667 885.333333 307.2 885.333333 512 716.8 885.333333 512 885.333333z";
static ABOUT2: &str = "M512 320m-42.666667 0a42.666667 42.666667 0 1 0 85.333334 0 42.666667 42.666667 0 1 0-85.333334 0Z";
static ABOUT3: &str = "M512 437.333333c-17.066667 0-32 14.933333-32 32v234.666667c0 17.066667 14.933333 32 32 32s32-14.933333 32-32V469.333333c0-17.066667-14.933333-32-32-32z";
static WARN: &str = "M849.12 928.704 174.88 928.704c-45.216 0-81.536-17.728-99.68-48.64-18.144-30.912-15.936-71.296 6.08-110.752L421.472 159.648c22.144-39.744 55.072-62.528 90.304-62.528s68.128 22.752 90.336 62.464l340.544 609.792c22.016 39.456 24.288 79.808 6.112 110.72C930.656 911.008 894.304 928.704 849.12 928.704zM511.808 161.12c-11.2 0-24.032 11.104-34.432 29.696L137.184 800.544c-10.656 19.136-13.152 36.32-6.784 47.168 6.368 10.816 22.592 17.024 44.48 17.024l674.24 0c21.92 0 38.112-6.176 44.48-17.024 6.336-10.816 3.872-28-6.816-47.136L546.24 190.816C535.872 172.224 522.976 161.12 511.808 161.12zM512 640c-17.664 0-32-14.304-32-32l0-288c0-17.664 14.336-32 32-32s32 14.336 32 32l0 288C544 625.696 529.664 640 512 640zM512 752.128m-48 0a1.5 1.5 0 1 0 96 0 1.5 1.5 0 1 0-96 0Z";
static INFO: &str = "M360 848.458h40V559.542H360c-22.092 0-40-17.908-40-40V424c0-22.092 17.908-40 40-40h224c22.092 0 40 17.908 40 40v424.458h40c22.092 0 40 17.908 40 40V984c0 22.092-17.908 40-40 40H360c-22.092 0-40-17.908-40-40v-95.542c0-22.092 17.908-40 40-40zM512 0C432.47 0 368 64.47 368 144s64.47 144 144 144 144-64.47 144-144S591.528 0 512 0z";
static SUCCESS: &str = "M939.126472 312.141136 939.126472 312.141136 449.642279 801.685705c-11.582803 11.605316-27.561729 18.733667-45.196365 18.733667-17.593703 0-33.549094-7.128351-45.131897-18.733667L82.546529 524.989849c-11.523451-11.533684-18.671245-27.528983-18.671245-45.090964 0-35.270295 28.595268-63.938218 63.866586-63.938218 17.633612 0 33.612539 7.188726 45.195342 18.721387l231.509724 231.562936 444.391183-444.452581c11.56336-11.531638 27.561729-18.649755 45.215808-18.649755 35.228339 0 63.866586 28.586059 63.866586 63.865563C957.920514 284.58964 950.792163 300.619732 939.126472 312.141136";
static CLEAN: &str = "M622 112c17.673 0 32 14.327 32 32l-0.001 139H879c17.673 0 32 14.327 32 32v164c0 17.673-14.327 32-32 32h-25.001L854 880c0 17.673-14.327 32-32 32H201c-17.673 0-32-14.327-32-32l-0.001-369H144c-17.673 0-32-14.327-32-32V315c0-17.673 14.327-32 32-32h224.999L369 144c0-17.673 14.327-32 32-32h221z m176 400H225v344h87.343V739.4c0-30.927 25.072-56 56-56V856h115.656L484 739.4c0-30.927 25.072-56 56-56l-0.001 172.6h115L655 739.4c0-30.927 25.072-56 56-56l-0.001 172.6H798V512z m49-165H176v100h671V347zM590 176H433v100h157V176z";

#[component]
pub fn header(
    mut link_list: Signal<LinkList>,
    mut filter_name: Signal<Option<String>>,
    mut current_tab: Signal<Tab>,
    mut msgbox: Signal<Option<(MsgIcon, Action)>>,
) -> Element {
    let mut maximized_icon = use_signal(|| MAXIMIZE1);
    rsx! {
        style { {include_str!("head.css")} },
        div {
            class: "header-container",
            div {
                class: "logo",
                img { src: LOGO_BASE64 },
            },
            div {
                class: "search-actions-container",
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        *msgbox.write() = Some((
                            MsgIcon::Warn(
                                t!("WARN_RESTORE_ALL").into_owned(),
                                t!("RESTORE_ALL_TOOLTIP").into_owned(),
                                t!("RESTORE").into_owned(),
                            ),
                            Action::RestoreAll
                        ));
                    },
                    background: "#B54646",
                    div {
                        class: "svg-wrapper",
                        svg {
                            view_box: "0 0 1024 1024",
                            path { fill: "#ffffff", d: RESTORE }
                        },
                    }
                    span { class:"text", {t!("RESTORE_ICON")} }
                    span { class:"tooltip", {t!("RESTORE_ALL_TOOLTIP")} }
                },
                input {
                    onmousedown: |event| event.stop_propagation(),
                    r#type: "text",
                    placeholder: t!("SEARCH").into_owned(),
                    oninput: move |event| {
                        if *current_tab.read() != Tab::Home {
                            *current_tab.write() = Tab::Home
                        };
                        let value = event.value();
                        if value.trim().is_empty() {
                            *filter_name.write() = None
                        } else {
                            *filter_name.write() = Some(value.trim().to_string());
                        };
                    }
                },
                button {
                    onmousedown: |event| event.stop_propagation(),
                    onclick: move |_| {
                        match change_all_shortcuts_icons(link_list) {
                            Ok(true) => {
                                notify(&t!("SUCCESS_CHANGE_ALL"));
                                if *current_tab.read() != Tab::Home {
                                    *current_tab.write() = Tab::Home
                                };
                            },
                            Ok(false) => println!("{}", t!("NOT_CHANGE_ALL")),
                            Err(err) => {
                                notify(&t!("ERROR_CHANGE_ALL"));
                                write_log(err.to_string()).expect("Failed to write 'change all shortcuts icons' log")
                            }
                        }
                    },
                    background: "#6148df",
                    div {
                        class: "svg-wrapper",
                        svg {
                            view_box: "0 0 24 24",
                            path { fill: "#ffffff", d: CHANGE }
                        },
                    }
                    span { class:"text", {t!("CHANGE")} }
                    span { class:"tooltip", {t!("CHANGE_ALL")} }
                }
            },
            div {
                class: "window-buttons",
                button {
                    onclick: move |_| window().set_minimized(true),
                    onmousedown: |event| event.stop_propagation(),
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: MINIMIZED }
                    },
                },
                button {
                    onclick: move |_| {
                        if window().is_maximized() {
                            window().set_maximized(false);
                            *maximized_icon.write() = MAXIMIZE1;
                        } else {
                            window().set_maximized(true);
                            *maximized_icon.write() = MAXIMIZE2;
                        };
                    },
                    onmousedown: |event| event.stop_propagation(),
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: maximized_icon }
                    },
                },
                button {
                    class: "close-button",
                    onclick: move |_| window().close(),
                    onmousedown: |event| event.stop_propagation(),
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: CLOSE1 },
                        path { d: CLOSE2 },
                    },
                }
            }
        }  
    }
}

#[component]
pub fn tabs(current_tab: Signal<Tab>) -> Element {
    let tab_svg = vec![
        (Tab::Home, (HOME, None, None)),
        (Tab::Tool, (TOOL, None, None), ),
        (Tab::History, (HISTORY1, Some(HISTORY2), None)),
        (Tab::Setting, (SETTING1, Some(SETTING2), None)),
        (Tab::About, (ABOUT1, Some(ABOUT2), Some(ABOUT3))),
    ]
    .into_iter()
    .map(|(tab, svg)| {
        let color = if *current_tab.read() == tab { "select-color" } else { "unselect-color" };
        (tab, svg, color)
    })
    .collect::<Vec<_>>();

    rsx!{
        style { {include_str!("tab.css")} },
        div {
            class: "tab-container",
            for (tab, (d1, d2, d3), color_class) in tab_svg.into_iter() {
                button {
                    class: color_class,
                    onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
                    onclick: move |_| *current_tab.write() = tab,
                    svg {
                        view_box: "0 0 1024 1024",
                        path { d: d1 },
                        if let Some(d2) = d2 {
                            path { d: d2 },
                        },
                        if let Some(d3) = d3 {
                            path { d: d3 },
                        },
                    }
                }
            }
        }
    }
}

#[component]
pub fn home(
    mut filter_name: Signal<Option<String>>,
    mut link_list: Signal<LinkList>,
    mut msgbox: Signal<Option<(MsgIcon, Action)>>,
    mut should_show_prop: Signal<bool>,
) -> Element {
    let filter_link_list_items = match filter_name.read().as_deref() {
        Some(name) => {
            link_list.read().items.clone()
                .into_iter()
                .enumerate()
                .filter(|(_, item)| item.name.to_lowercase().contains(name))
                .map(|(index, item)| (item, Some(index)))
                .collect::<Vec<(LinkProp, Option<usize>)>>()
        },
        None => link_list.read().items.clone()
            .into_iter()
            .map(|item| (item, None))
            .collect(),
    };

    rsx! {
        style { {include_str!("icon.css")} },
        div {
            class: "icon-container",
            onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
            for (filter_index, (item, index)) in filter_link_list_items.into_iter().enumerate() {
                if let Some(index) = index {
                    components::icon_button{ item, index, link_list },
                } else {
                    components::icon_button{ item, index: filter_index, link_list },
                }
            }
        },
        div {
            class: "icon-modify-container ",
            components::icon_modify{ link_list, msgbox, should_show_prop }
        }
    }
}

#[component]
pub fn icon_button(item: LinkProp, index: usize, mut link_list: Signal<LinkList>) -> Element {
    rsx! {
        button {
            class: "icon-button",
            ondoubleclick: move |_| {
                match change_single_shortcut_icon(link_list) {
                    Ok(Some(name)) => notify(&format!("{}: {}", t!("SUCCESS_CHANGE_ONE"), name)),
                    _ => (),
                };
            },
            onclick: move |_| link_list.write().state.select = Some(index),
            div {
                class: "img-container",
                img { src: item.icon.clone() },
                span { {item.name.clone()} },
            }
        },
    }
}

#[component]
pub fn icon_modify(
    mut link_list: Signal<LinkList>,
    mut msgbox: Signal<Option<(MsgIcon, Action)>>,
    mut should_show_prop: Signal<bool>,
) -> Element {
    if let Some(index) = link_list.read().state.select {
        let link_target_path = &link_list.read().items[index].target_path;
        let link_target_dir = link_list.read().items[index].target_dir.clone();
        let link_icon_path = link_list.read().items[index].icon_location.clone();

        let check_path_exists = |path: &str| -> &str {
            if Path::new(path).exists() {
                "allowed"
            } else {
                "not-allowed"
            }
        };

        let should_restore_allow = check_path_exists(link_target_path);
        let should_open_target_dir_allow = check_path_exists(&link_target_dir);
        let should_open_icon_dir_allow = check_path_exists(&link_icon_path);

        rsx!{
            style { {include_str!("modify.css")} },
            div {
                class: "contrast-icon-container",
                
            },
            div {
                class: "modify-icon-container",
                onmousedown: |event| event.stop_propagation(),
                button {
                    class: "allowed",
                    onclick: move |_| {
                        match change_single_shortcut_icon(link_list) {
                            Ok(Some(name)) => notify(&format!("{}: {}", t!("SUCCESS_CHANGE_ONE"), name)),
                            _ => (),
                        };
                    },
                    span { {t!("CHANGE_ONE")} }
                }
                button {
                    class: should_restore_allow,
                    onclick: move |_| {
                        if should_restore_allow == "allowed" {
                            *msgbox.write() = Some((
                                MsgIcon::Warn(
                                    t!("WARN_RESTORE_ONE").into_owned(),
                                    t!("RESTORE_ONE_TOOLTIP").into_owned(),
                                    t!("RESTORE").into_owned(),
                                ),
                                Action::RestoreOne
                            ));
                        }
                    },
                    span { {t!("RESTORE_ONE")} },
                }
                button {
                    class: should_open_target_dir_allow,
                    onclick: move |_| {
                        if should_open_target_dir_allow == "allowed" {
                            if let Err(err) = opener::open(&link_target_dir) {
                                write_log(format!("Failed to open {link_target_dir}: {err}")).expect("Failed to write the log");
                            };
                        }
                    },
                    span { {t!("TARGET_DIR")} }
                }
                button {
                    class: should_open_icon_dir_allow,
                    onclick: move |_| {
                        if should_open_icon_dir_allow == "allowed" {
                            let link_icon_dir_path = Path::new(&link_icon_path).parent();
                            if let Some(path) = link_icon_dir_path {
                                if let Err(err) = opener::open(path) {
                                    write_log(format!("Failed to open {}: {err}", path.display())).expect("Failed to write the log");
                                };
                            };
                        }
                    },
                    span { {t!("ICON_DIR")} }
                }
                button {
                    class: "allowed",
                    onclick: move |_| {
                        *should_show_prop.write() = true;
                    },
                    span { {t!("VIEW_PROPERTIES")} }
                }
            }
        }
    } else {
        rsx!()
    }
}

#[component]
pub fn status(link_list: Signal<LinkList>) -> Element {
    let icon_from = link_list.read().source.name();
    let items = link_list.read().items.clone();
    let total = items.len();
    let changed_numbers = items.iter().filter(|i| i.status == Status::Changed).count();
    let unchanged_numbers = total - changed_numbers;
    let status_texts = [
        format!("{}: {icon_from}", t!("LOCATION")),
        format!("{}: {total}", t!("TOTAL")),
        format!("{}: {changed_numbers}", t!("CHANGED")),
        format!("{}: {unchanged_numbers}", t!("UNCHANGED")),
    ];

    rsx!{
        style { {include_str!("status.css")} },
        div {
            class: "status-container",
            for (index, text) in status_texts.iter().enumerate() {
                span { { text.clone()} },
                if index != status_texts.len() - 1 {
                    span { "〡" },
                }
            }
        }
    }
}

#[component]
pub fn msg_box(
    mut msgbox: Signal<Option<(MsgIcon, Action)>>,
    mut link_list: Signal<LinkList>,
    mut current_tab: Signal<Tab>,
) -> Element {
    rsx!{
        if let Some((msg_icon, action)) = msgbox.read().clone() {
            style { {include_str!("msgbox.css")} },
            div {
                class: "msgbox-container",
                onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
                div {
                    class: "msgbox-modal",
                    div {
                        class: "image",
                        background_color: msg_icon.svg_back_color(),
                        svg {
                            view_box: "0 0 1024 1024",
                            path {
                                fill: msg_icon.svg_fill(),
                                d: msg_icon.svg_d(),
                            },
                        }
                    }
                    div {
                        class: "content",
                        if let Some(title) = msg_icon.title() {
                            span {
                                class: "title",
                                {title}
                            }
                        }
                        p {
                            class: "message",
                            {msg_icon.messages()}
                        }
                    }
                    button {
                        onclick: move |_|  {
                            match action {
                                Action::RestoreAll => {
                                    if let Err(err) = restore_all_shortcuts_icons(link_list) {
                                        notify(&t!("ERROR_RESTORE_ALL"));
                                        write_log(err.to_string()).expect("Failed to write log")
                                    } else {
                                        notify(&t!("SUCCESS_RESTORE_ALL"));
                                        if *current_tab.read() != Tab::Home {
                                            *current_tab.write() = Tab::Home
                                        };
                                    };
                                },
                                Action::RestoreOne => {
                                    if let Ok(resotre) = restore_single_shortcut_icon(link_list) {
                                        if let Some(name) = resotre {
                                            notify(&format!("{}: {}", t!("SUCCESS_RESTORE_ONE"), name));
                                        }
                                    } else {
                                        notify(&t!("ERROR_RESTORE_ONE"));
                                    }
                                },
                                _ => ()
                            }

                            *msgbox.write() = None
                        },
                        class: "confirm",
                        background_color: msg_icon.svg_fill(),
                        {msg_icon.comfirm_name()}
                    }
                    if action != Action::None {
                        button {
                            onclick: move |_| {
                                *msgbox.write() = None
                            },
                            class: "cancel",
                            {t!("CANCEL")}
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Action {
    RestoreOne,
    RestoreAll,
    None
}

#[derive(PartialEq, Clone)]
pub enum MsgIcon {
    Warn(String, String, String), // messages, title, confrim_name
    Info(String), // messages
    Success(String), // messages
    Clean(String, String), // messages, title
}

impl MsgIcon {
    pub fn messages(&self) -> String {
        match self {
            MsgIcon::Warn(m, _, _) 
            | MsgIcon::Info(m)
            | MsgIcon::Success(m)
            | MsgIcon::Clean(m, _) => m.to_owned(),
        }
    }

    pub fn svg_d(&self) -> &str {
        match self {
            MsgIcon::Warn(_, _, _) => WARN,
            MsgIcon::Info(_) => INFO,
            MsgIcon::Success(_) => SUCCESS,
            MsgIcon::Clean(_, _) => CLEAN
        }
    }

    pub fn svg_fill(&self) -> &str {
        match self {
            MsgIcon::Warn(_, _, _) => "#DC2626", 
            MsgIcon::Info(_) => "#2196F3",
            MsgIcon::Success(_) => "#E2FEEE",
            MsgIcon::Clean(_, _) => "#FFD25F",
        }
    }

    pub fn svg_back_color(&self) -> &str {
        match self {
            MsgIcon::Warn(_, _, _) => "#FEE2E2",
            MsgIcon::Info(_) => "#85C2F3",
            MsgIcon::Success(_) => "#1AA06D",
            MsgIcon::Clean(_, _) => "#FFE78B"
        }
    }

    pub fn title(&self) -> Option<String> {
        match self {
            MsgIcon::Warn(_, title, _) | MsgIcon::Clean(_, title) => Some(title.to_owned()),
            _ => None,
        }
    }
    pub fn comfirm_name(&self) -> String {
        match self {
            MsgIcon::Warn(_, _, name) | MsgIcon::Clean(_, name) => name.to_owned(),
            MsgIcon::Info(_) | MsgIcon::Success(_) => t!("COMFIRM").into_owned(),
        }
    }
}

#[component]
pub fn properties(
    mut link_list: Signal<LinkList>,
    mut should_show_prop: Signal<bool>
) -> Element {
    fn rsx_info(label:&str, value: &str) -> Element {
        rsx! {
            div {
                class: "item",
                span { "{label}: {value}" },
            }
        }
    }

    if *should_show_prop.read() {
        if let Some(index) = link_list.read().state.select {   
            let item = &link_list.read().items[index];             
            rsx!{
                style { {include_str!("properties.css")} },
                div {
                    class: "properties-container",
                    div {
                        class: "properties-modal",
                        div {
                            class: "head",
                            span { 
                                {item.name.clone()}
                            },
                            button {
                                onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
                                onclick: move |_| *should_show_prop.write() = false,
                                "X"
                            }
                        }
                        div {
                            class: "items",
                            onmousedown: |event| event.stop_propagation(), // 屏蔽拖拽
                            {rsx_info(&t!("FILE_PATH"), &item.path)}
                            {rsx_info(&t!("TARGET_PATH"), &item.target_path)}
                            {rsx_info(&t!("ICON_PATH"), &item.icon_location)}
                            {rsx_info(&t!("ARGUMENTS"), &item.arguments)}
                            {rsx_info(&t!("FILE_SIZE"), &item.file_size)}
                            {rsx_info(&t!("CREATED_AT"), &item.created_at)}
                            {rsx_info(&t!("UPDATED_AT"), &item.updated_at)}
                            {rsx_info(&t!("ACCESSED_AT"), &item.accessed_at)}
                        }
                    }
                }
            }
        } else {
            rsx!()
        }
    } else {
        rsx!()   
    }
}