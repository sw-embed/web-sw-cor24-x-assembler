//! IndexedDB wrapper for binary image storage.
//!
//! One database (`web-sw-cor24-x-assembler`) with one object store
//! (`images`) keyed by string and valued as `Uint8Array`. Used by
//! the SPI SD card + NOR flash panels to persist user-uploaded
//! disk images across page reloads — localStorage is too small for
//! the 4 MiB W25Q32 image and not the right tool for binary blobs
//! generally.
//!
//! All operations are async; callers should `spawn_local` the
//! future. Errors silently no-op (the panel falls back to in-memory
//! state on IDB failure — same fallback as a fresh first visit).

use js_sys::{Promise, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Event, IdbDatabase, IdbObjectStoreParameters, IdbOpenDbRequest, IdbRequest, IdbTransactionMode,
    IdbVersionChangeEvent,
};

const DB_NAME: &str = "web-sw-cor24-x-assembler";
const STORE_NAME: &str = "images";
const DB_VERSION: u32 = 1;

/// Open the database, creating the `images` object store on first
/// use. Each call opens fresh — IDB caches the connection internally
/// so this is cheap.
async fn open_db() -> Result<IdbDatabase, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let factory = window
        .indexed_db()?
        .ok_or_else(|| JsValue::from_str("no IDB"))?;
    let request: IdbOpenDbRequest = factory.open_with_u32(DB_NAME, DB_VERSION)?;

    // First-run schema setup: create the object store. Fired
    // before the success event so the store exists by the time the
    // first transaction opens.
    let onupgrade = Closure::wrap(Box::new(move |e: IdbVersionChangeEvent| {
        let target = e.target().expect("upgrade event target");
        let req: IdbOpenDbRequest = target.dyn_into().expect("upgrade event from IdbOpenDbRequest");
        if let Ok(result) = req.result()
            && let Ok(db) = result.dyn_into::<IdbDatabase>()
        {
            // upgradeneeded only fires on fresh DBs (or version bumps);
            // create_object_store unconditionally is safe -- a duplicate
            // create would return Err which we ignore.
            let _ = db.create_object_store_with_optional_parameters(
                STORE_NAME,
                &IdbObjectStoreParameters::new(),
            );
        }
    }) as Box<dyn FnMut(IdbVersionChangeEvent)>);
    request.set_onupgradeneeded(Some(onupgrade.as_ref().unchecked_ref()));
    onupgrade.forget();

    let result = JsFuture::from(request_to_promise(request.unchecked_ref())).await?;
    let db: IdbDatabase = result.dyn_into()?;
    Ok(db)
}

/// Bridge an `IdbRequest` to a JS `Promise` by wiring its
/// `onsuccess` / `onerror` events. Caller awaits via
/// `JsFuture::from(...)`. The resolved value is the request's
/// `result` at success time.
fn request_to_promise(request: &IdbRequest) -> Promise {
    let req = request.clone();
    Promise::new(&mut |resolve, reject| {
        let req_for_success = req.clone();
        let onsuccess = Closure::wrap(Box::new(move |_e: Event| {
            let result = req_for_success.result().unwrap_or(JsValue::NULL);
            let _ = resolve.call1(&JsValue::NULL, &result);
        }) as Box<dyn FnMut(Event)>);
        let onerror = Closure::wrap(Box::new(move |_e: Event| {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("IDB request failed"));
        }) as Box<dyn FnMut(Event)>);
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        req.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        // The closures must outlive the JS-side event firing; the
        // IDB request itself is short-lived (single resolution),
        // so forget() is correct here -- we leak two ~64-byte
        // closures per call, but each call is per-user-action.
        onsuccess.forget();
        onerror.forget();
    })
}

/// Read the bytes stored at `key`, or `None` if absent / on error.
pub async fn get(key: &str) -> Option<Vec<u8>> {
    let db = open_db().await.ok()?;
    let tx = db
        .transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readonly)
        .ok()?;
    let store = tx.object_store(STORE_NAME).ok()?;
    let request = store.get(&JsValue::from_str(key)).ok()?;
    let result = JsFuture::from(request_to_promise(&request)).await.ok()?;
    if result.is_null() || result.is_undefined() {
        return None;
    }
    let array: Uint8Array = result.dyn_into().ok()?;
    let mut buf = vec![0u8; array.length() as usize];
    array.copy_to(&mut buf);
    Some(buf)
}

/// Write `bytes` to `key`, overwriting any prior value. Errors are
/// swallowed -- the panel state stays correct in-memory either way.
pub async fn put(key: &str, bytes: &[u8]) {
    let Ok(db) = open_db().await else { return };
    let Ok(tx) = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)
    else {
        return;
    };
    let Ok(store) = tx.object_store(STORE_NAME) else {
        return;
    };
    let array = Uint8Array::new_with_length(bytes.len() as u32);
    array.copy_from(bytes);
    let Ok(request) = store.put_with_key(&array, &JsValue::from_str(key)) else {
        return;
    };
    let _ = JsFuture::from(request_to_promise(&request)).await;
}

/// Delete the value at `key`. No-op if not present.
pub async fn delete(key: &str) {
    let Ok(db) = open_db().await else { return };
    let Ok(tx) = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)
    else {
        return;
    };
    let Ok(store) = tx.object_store(STORE_NAME) else {
        return;
    };
    let Ok(request) = store.delete(&JsValue::from_str(key)) else {
        return;
    };
    let _ = JsFuture::from(request_to_promise(&request)).await;
}
