use crate::controller::authentication::*;
use crate::controller::data::*;
use crate::controller::effects::*;
use crate::controller::friends::*;
use crate::controller::inventory::*;
use crate::controller::mail::*;
use crate::controller::stats::*;
use crate::controller::user::*;
use crate::controller::shop::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    create_user,
    login,
    remove_friend,
    add_friend_request,
    handle_friend_request,
    select_item,
    add_playtime,
    add_fish,
    create_mail,
    delete_mail,
    change_read_state,
    change_archive_state,
    use_item,
    destroy_item,
    add_effect,
    remove_expired_effects,
    cleanup_all_expired_effects,
    retreive_player_data,
    buy_item,
))]
pub struct ApiDoc;
