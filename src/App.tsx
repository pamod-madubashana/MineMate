import { BrowserRouter, Routes, Route } from "react-router-dom";
import TopNavBar from "./components/shell/TopNavBar";
import SideNavBar from "./components/shell/SideNavBar";
import Dashboard from "./components/dashboard/Dashboard";
import ChatLog from "./components/chat/ChatLog";
import ConfigPanel from "./components/config/ConfigPanel";
import TaskQueue from "./components/tasks/TaskQueue";
import Blueprints from "./components/blueprints/Blueprints";

function App() {
  return (
    <BrowserRouter>
      <div className="app-shell">
        <TopNavBar />
        <div className="main-layout">
          <SideNavBar />
          <main className="content-area">
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/chat" element={<ChatLog />} />
              <Route path="/config" element={<ConfigPanel />} />
              <Route path="/tasks" element={<TaskQueue />} />
              <Route path="/blueprints" element={<Blueprints />} />
            </Routes>
          </main>
        </div>
      </div>
    </BrowserRouter>
  );
}

export default App;
