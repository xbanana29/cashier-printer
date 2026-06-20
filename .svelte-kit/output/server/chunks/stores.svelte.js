import "clsx";
const toast = { message: null };
let toastTimer = null;
function showToast(text, type = "success") {
  if (toastTimer) clearTimeout(toastTimer);
  toast.message = { text, type };
  toastTimer = setTimeout(
    () => {
      toast.message = null;
      toastTimer = null;
    },
    3e3
  );
}
function showError(err) {
  const msg = err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
  showToast(msg, "error");
}
export {
  showError as a,
  showToast as s,
  toast as t
};
