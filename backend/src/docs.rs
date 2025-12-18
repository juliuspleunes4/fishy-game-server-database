use crate::controller::authentication::*;
use crate::controller::competitions::*;
use crate::controller::data::*;
use crate::controller::effects::*;
use crate::controller::friends::*;
use crate::controller::inventory::*;
use crate::controller::mail::*;
use crate::controller::stats::*;
use crate::controller::user::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    create_user,
    get_user,
    login,

    remove_friend,
    add_friend_request,
    handle_friend_request,

    select_item,
    add_xp,
    change_bucks,
    change_coins,
    add_playtime,
    add_fish,

    create_mail,
    delete_mail,
    change_read_state,
    change_archive_state,

    add_or_update_item,
    destroy_item,

    add_effect,
    remove_expired_effects,
    cleanup_all_expired_effects,

    create_competition,
    get_active_competitions,
    get_upcoming_competitions,
    get_competition,
    update_score,
    generate_competitions,
    cleanup_competitions,

    retreive_player_data,
))]
pub struct ApiDoc;
