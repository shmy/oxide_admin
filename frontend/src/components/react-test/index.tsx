import { Button, Segmented } from "antd";
import { useEffect } from "react";

const ReactTest = () => {
  useEffect(() => {
    console.log("ReactTest onMounted");
    return () => {
      console.log("ReactTest onUnmounted");
    };
  }, []);

  return (
    <div>
      <Button
        type="primary"
        onClick={() => {
          console.log("click");
        }}
      >
        This is button
      </Button>
      <br />
      <Segmented<string>
        options={["Daily", "Weekly", "Monthly", "Quarterly", "Yearly"]}
        onChange={(value) => {
          console.log(value); // string
        }}
      />
    </div>
  );
};

export default ReactTest;
