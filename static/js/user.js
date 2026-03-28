// 1. 定义主题应用函数
const applyTheme = () => {
    const savedTheme = localStorage.getItem("theme") || "system";
    const isSystemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    
    // 核心逻辑：如果是 dark 或者 (system 且系统是 dark)，就加上 .dark
    const shouldBeDark = savedTheme === "dark" || (savedTheme === "system" && isSystemDark);
    
    document.documentElement.classList.toggle("dark", shouldBeDark);
};

// 2. 监听系统主题实时变化
// 注意：这里使用同一个媒体查询对象
const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");
darkQuery.addEventListener("change", (e) => {
    // 只有当用户设置为 "system" 时，才响应系统级的切换
    if ((localStorage.getItem("theme") || "system") === "system") {
        document.documentElement.classList.toggle("dark", e.matches);
    }
});

// 3. 初始加载执行一次
applyTheme();

// 4. 全局切换函数 (供菜单调用)
window.setTheme = function (theme) {
    localStorage.setItem("theme", theme);
    applyTheme();
    console.log(`主题已切换至: ${theme}`);
};

// 5. DOM 交互逻辑
document.addEventListener('DOMContentLoaded', () => {
    const themeMenu = document.getElementById("theme-menu");
    const themeBtn = document.getElementById("theme-btn");

    if (themeBtn && themeMenu) {
        themeBtn.addEventListener("click", () => {
            themeMenu.open ? themeMenu.close() : themeMenu.show();
        });
        
        themeMenu.defaultFocus = 'NONE';
        themeMenu.addEventListener("close-menu", (event) => {
            const menuItem = event.detail.initiator;
            const mode = menuItem.getAttribute("data-mode");
            if (mode) window.setTheme(mode);
        });
    }
});